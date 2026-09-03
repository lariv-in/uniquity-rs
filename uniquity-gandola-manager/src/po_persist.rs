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
use crate::po_lines::{parse_po_lines_json, po_lines_form_json, replace_po_lines};
use crate::po_payment_term::{
    payment_term_lines_form_json_for_po_term, upsert_purchase_order_payment_term_lines,
};
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
    let site =
        lariv_rs::web::opt_or_log(SiteEntity::find_by_id(site_id).one(db).await, "find by id")
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
    lariv_rs::web::opt_or_log(query.one(db).await, "db find one").is_some()
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
                if let Err(rollback_err) =
                    PurchaseOrderEntity::delete_by_id(saved.id).exec(db).await
                {
                    tracing::error!(
                        error = %rollback_err,
                        po_id = saved.id,
                        "failed to rollback purchase order after line replace error"
                    );
                }
                if let Err(rollback_err) = PurchaseOrderPaymentTermEntity::delete_by_id(term.id)
                    .exec(db)
                    .await
                {
                    tracing::error!(
                        error = %rollback_err,
                        term_id = term.id,
                        "failed to rollback payment term after line replace error"
                    );
                }
                return Err(e);
            }
            Ok(saved)
        }
        Err(e) => {
            if let Err(rollback_err) = PurchaseOrderPaymentTermEntity::delete_by_id(term.id)
                .exec(db)
                .await
            {
                tracing::error!(
                    error = %rollback_err,
                    term_id = term.id,
                    "failed to rollback payment term after purchase order insert error"
                );
            }
            Err(e.to_string())
        }
    }
}

pub async fn purchase_order_form_from_model(
    db: &DatabaseConnection,
    po: &purchase_order::Model,
    tz: &str,
) -> PurchaseOrderForm {
    PurchaseOrderForm {
        number: po.number.clone(),
        date: lariv_rs::datetime::format_date(po.date),
        customer_id: po.customer_id,
        site_id: po.site_id,
        file_id: po
            .file_id
            .filter(|&id| id > 0)
            .map(|id| id.to_string())
            .unwrap_or_default(),
        payment_term_lines_json: payment_term_lines_form_json_for_po_term(
            db,
            po.payment_term_id,
            tz,
        )
        .await,
        po_lines_json: po_lines_form_json(db, po.id).await,
        billing_address: po.billing_address.clone().unwrap_or_default(),
        shipping_address: po.shipping_address.clone().unwrap_or_default(),
    }
}

pub async fn persist_updated_purchase_order(
    db: &DatabaseConnection,
    existing: &purchase_order::Model,
    form: &PurchaseOrderForm,
    tz: &str,
) -> Result<purchase_order::Model, String> {
    let (date, number) = validate_purchase_order_form(form)?;
    let (site_id, customer_id) =
        resolve_site_and_customer(db, form.site_id, form.customer_id).await?;
    if purchase_order_number_taken(db, &number, Some(existing.id)).await {
        return Err("number must be unique".into());
    }
    let lines = parse_po_lines_json(&form.po_lines_json)?;
    let term_lines = parse_payment_term_lines_json(&form.payment_term_lines_json)?;
    let term =
        upsert_purchase_order_payment_term_lines(db, existing.payment_term_id, &term_lines, tz)
            .await?;
    let now = Utc::now();
    let model = purchase_order::ActiveModel {
        id: Set(existing.id),
        number: Set(number),
        date: Set(date),
        customer_id: Set(customer_id),
        site_id: Set(site_id),
        file_id: Set(parse_optional_i64(&form.file_id)),
        payment_term_id: Set(Some(term.id)),
        billing_address: Set(opt_string(form.billing_address.clone())),
        shipping_address: Set(opt_string(form.shipping_address.clone())),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let saved = model.update(db).await.map_err(|e| e.to_string())?;
    replace_po_lines(db, saved.id, &lines).await?;
    Ok(saved)
}

pub async fn delete_purchase_order(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    if id <= 0 {
        return Err("delete_purchase_order requires a positive purchase order id".into());
    }
    let existing = PurchaseOrderEntity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("purchase order {id} not found"))?;
    let term_id = existing.payment_term_id;
    PurchaseOrderEntity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(term_id) = term_id {
        if let Err(e) = PurchaseOrderPaymentTermEntity::delete_by_id(term_id)
            .exec(db)
            .await
        {
            tracing::error!(
                error = %e,
                term_id,
                po_id = id,
                "failed to delete purchase order payment term after PO delete"
            );
        }
    }
    Ok(())
}
