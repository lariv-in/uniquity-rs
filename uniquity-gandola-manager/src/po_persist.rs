//! Shared purchase-order create/update persistence used by HTTP handlers and import CLI.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use lariv_rs::plugins::finance_invoices::logic::parse_payment_term_lines_json;

use crate::entities::{
    PurchaseOrderPaymentTermEntity, SiteEntity,
    purchase_order::{self, Entity as PurchaseOrderEntity},
};
use crate::forms::PurchaseOrderForm;
use crate::po_lines::{parse_po_lines_json, replace_po_lines};
use crate::po_payment_term::upsert_purchase_order_payment_term_lines;
use crate::scope::{opt_string, parse_optional_i64};

fn parse_date(s: &str) -> Result<chrono::NaiveDate, &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Err("date is required");
    }
    lariv_rs::datetime::parse_date(s).ok_or("invalid date")
}

pub fn validate_purchase_order_form(
    form: &PurchaseOrderForm,
) -> Result<(chrono::NaiveDate, String), String> {
    let number = form.number.trim();
    if number.is_empty() {
        return Err("number is required".into());
    }
    if form.customer_id <= 0 {
        return Err("select a customer".into());
    }
    if form.site_id <= 0 {
        return Err("select a site".into());
    }
    let date = parse_date(&form.date).map_err(|e| e.to_string())?;
    parse_payment_term_lines_json(&form.payment_term_lines_json)?;
    parse_po_lines_json(&form.po_lines_json)?;
    Ok((date, number.to_string()))
}

pub async fn resolve_site_and_customer(
    db: &DatabaseConnection,
    site_id: i64,
    customer_id: i64,
) -> Result<(i64, i64), String> {
    if site_id <= 0 {
        return Err("select a site".into());
    }
    let site = SiteEntity::find_by_id(site_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .ok_or_else(|| "select a site".to_string())?;
    if customer_id > 0 && site.customer_id != customer_id {
        return Err("site does not belong to the selected customer".into());
    }
    Ok((site.id, site.customer_id))
}

pub async fn purchase_order_number_taken(
    db: &DatabaseConnection,
    number: &str,
    except_id: Option<i64>,
) -> bool {
    let mut query = PurchaseOrderEntity::find().filter(purchase_order::Column::Number.eq(number));
    if let Some(id) = except_id {
        query = query.filter(purchase_order::Column::Id.ne(id));
    }
    query.one(db).await.ok().flatten().is_some()
}

pub async fn persist_new_purchase_order(
    db: &DatabaseConnection,
    form: &PurchaseOrderForm,
    tz: &str,
) -> Result<purchase_order::Model, String> {
    let (date, number) = validate_purchase_order_form(form)?;
    let (site_id, customer_id) =
        resolve_site_and_customer(db, form.site_id, form.customer_id).await?;
    if purchase_order_number_taken(db, &number, None).await {
        return Err("number must be unique".into());
    }
    let lines = parse_po_lines_json(&form.po_lines_json)?;
    let term_lines = parse_payment_term_lines_json(&form.payment_term_lines_json)?;
    let term = upsert_purchase_order_payment_term_lines(db, None, &term_lines, tz).await?;
    let now = Utc::now();
    let file_id = parse_optional_i64(&form.file_id);
    let model = purchase_order::ActiveModel {
        number: Set(number),
        date: Set(date),
        customer_id: Set(customer_id),
        site_id: Set(site_id),
        file_id: Set(file_id),
        payment_term_id: Set(Some(term.id)),
        billing_address: Set(opt_string(form.billing_address.clone())),
        shipping_address: Set(opt_string(form.shipping_address.clone())),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    match model.insert(db).await {
        Ok(saved) => {
            if let Err(e) = replace_po_lines(db, saved.id, &lines).await {
                let _ = PurchaseOrderEntity::delete_by_id(saved.id).exec(db).await;
                let _ = PurchaseOrderPaymentTermEntity::delete_by_id(term.id)
                    .exec(db)
                    .await;
                return Err(e);
            }
            Ok(saved)
        }
        Err(e) => {
            let _ = PurchaseOrderPaymentTermEntity::delete_by_id(term.id)
                .exec(db)
                .await;
            Err(e.to_string())
        }
    }
}
