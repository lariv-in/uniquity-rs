//! Create draft invoices from every purchase order on a site.

use std::collections::HashMap;

use chrono::NaiveTime;
use lariv_rs::db::trigram;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;

use lariv_rs::plugins::finance_common::decimal::decimal_display;
use lariv_rs::plugins::finance_invoices::entities::draft_invoice::{
    self, Entity as DraftInvoiceEntity,
};
use lariv_rs::plugins::finance_invoices::entities::posted_invoice::{
    self, Entity as PostedInvoiceEntity,
};
use lariv_rs::plugins::finance_invoices::logic::draft::DraftLinePending;
use lariv_rs::plugins::finance_invoices::logic::{
    CreateDraftInput, create_draft_invoice, parse_payment_term_lines_json,
};
use lariv_rs::plugins::finance_products::entities::product::{self, Entity as ProductEntity};

use crate::entities::purchase_order;
use crate::entities::purchase_order_line::{self, Entity as PurchaseOrderLineEntity};
use crate::entities::site::{self, Entity as SiteEntity};
use crate::po_payment_term::payment_term_lines_form_json_for_po_term;
use crate::scope::{link_site_invoice, load_purchase_orders_for_site};

#[derive(Debug, Clone, Serialize)]
pub struct SiteSummary {
    pub id: i64,
    pub site_id: Option<String>,
    pub name: String,
    pub customer_id: i64,
    pub address: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PurchaseOrderLineSummary {
    pub id: i64,
    pub item_code: String,
    pub description: String,
    pub quantity: String,
    pub rate: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PurchaseOrderSummary {
    pub id: i64,
    pub number: String,
    pub date: String,
    pub customer_id: i64,
    pub already_invoiced: bool,
    pub existing_invoice_id: Option<i64>,
    pub lines: Vec<PurchaseOrderLineSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceAction {
    pub purchase_order_id: i64,
    pub purchase_order_number: String,
    pub invoice_id: Option<i64>,
    pub invoice_number: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateInvoicesResult {
    pub site_id: i64,
    pub site_name: String,
    pub dry_run: bool,
    pub created: Vec<InvoiceAction>,
    pub skipped: Vec<InvoiceAction>,
    pub errors: Vec<InvoiceAction>,
}

pub async fn find_site(
    db: &DatabaseConnection,
    site_id: Option<i64>,
    name: Option<&str>,
) -> Result<site::Model, String> {
    if let Some(id) = site_id.filter(|id| *id > 0) {
        return SiteEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("site #{id} not found"));
    }
    let needle = name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "provide site_id or site_name".to_string())?;
    let sites = trigram::search::<SiteEntity, _>(
        db,
        &[site::Column::Name, site::Column::SiteId],
        needle,
        50,
    )
    .await
    .map_err(|e| e.to_string())?;
    let exact: Vec<_> = sites
        .iter()
        .filter(|s| site_matches_needle(s, needle))
        .cloned()
        .collect();
    if exact.len() == 1 {
        return Ok(exact.into_iter().next().expect("one exact site"));
    }
    if exact.len() > 1 {
        return Err(format!(
            "multiple sites matching {:?}: {}",
            needle,
            site_id_list(&exact)
        ));
    }
    match sites.len() {
        1 => Ok(sites.into_iter().next().expect("one partial site")),
        0 => Err(format!("no site matching {needle:?}")),
        _ => Err(format!(
            "multiple sites match {needle:?}: {}",
            site_id_list(&sites)
        )),
    }
}

fn site_matches_needle(site: &site::Model, needle: &str) -> bool {
    site.name.eq_ignore_ascii_case(needle)
        || site
            .site_id
            .as_deref()
            .is_some_and(|id| id.eq_ignore_ascii_case(needle))
}

pub fn site_summary(site: &site::Model) -> SiteSummary {
    SiteSummary {
        id: site.id,
        site_id: site.site_id.clone(),
        name: site.name.clone(),
        customer_id: site.customer_id,
        address: site.address.clone(),
        status: site.status.as_str().to_string(),
    }
}

pub async fn list_site_purchase_orders(
    db: &DatabaseConnection,
    site_id: i64,
) -> Result<(site::Model, Vec<PurchaseOrderSummary>), String> {
    let site = find_site(db, Some(site_id), None).await?;
    let pos = load_purchase_orders_for_site(db, site.id).await;
    let mut out = Vec::with_capacity(pos.len());
    for po in pos {
        out.push(summarize_purchase_order(db, &po).await?);
    }
    Ok((site, out))
}

pub async fn create_invoices_for_site(
    db: &DatabaseConnection,
    site_id: Option<i64>,
    site_name: Option<&str>,
    timezone: &str,
    dry_run: bool,
) -> Result<CreateInvoicesResult, String> {
    let site = find_site(db, site_id, site_name).await?;
    let products = load_product_index(db).await?;
    let pos = load_purchase_orders_for_site(db, site.id).await;
    let mut result = CreateInvoicesResult {
        site_id: site.id,
        site_name: site.name.clone(),
        dry_run,
        created: Vec::new(),
        skipped: Vec::new(),
        errors: Vec::new(),
    };
    if pos.is_empty() {
        result.errors.push(InvoiceAction {
            purchase_order_id: 0,
            purchase_order_number: String::new(),
            invoice_id: None,
            invoice_number: String::new(),
            reason: Some("site has no purchase orders".into()),
        });
        return Ok(result);
    }

    for po in pos {
        match invoice_one_purchase_order(db, &site, &po, &products, timezone, dry_run).await {
            Ok(Outcome::Created(action)) => result.created.push(action),
            Ok(Outcome::Skipped(action)) => result.skipped.push(action),
            Err(action) => result.errors.push(action),
        }
    }
    Ok(result)
}

enum Outcome {
    Created(InvoiceAction),
    Skipped(InvoiceAction),
}

async fn invoice_one_purchase_order(
    db: &DatabaseConnection,
    site: &site::Model,
    po: &purchase_order::Model,
    products: &ProductIndex,
    timezone: &str,
    dry_run: bool,
) -> Result<Outcome, InvoiceAction> {
    if let Some(existing_id) = existing_invoice_id(db, &po.number)
        .await
        .map_err(|reason| action(po, None, Some(reason)))?
    {
        return Ok(Outcome::Skipped(action(
            po,
            Some(existing_id),
            Some("invoice already exists for this purchase order number".into()),
        )));
    }

    let lines = load_po_lines(db, po.id)
        .await
        .map_err(|reason| action(po, None, Some(reason)))?;
    if lines.is_empty() {
        return Err(action(po, None, Some("purchase order has no lines".into())));
    }

    let mut pending = Vec::with_capacity(lines.len());
    let mut missing = Vec::new();
    for line in &lines {
        match products.resolve(&line.item_code) {
            Some(product_id) => pending.push(DraftLinePending {
                product_id,
                rate: Some(decimal_display(line.rate)),
                quantity: decimal_display(line.quantity),
                tax_ids: None,
            }),
            None => missing.push(line.item_code.clone()),
        }
    }
    if !missing.is_empty() {
        return Err(action(
            po,
            None,
            Some(format!(
                "no product matches item_code(s): {}",
                missing.join(", ")
            )),
        ));
    }

    let payment_term_json =
        payment_term_lines_form_json_for_po_term(db, po.payment_term_id, timezone).await;
    let payment_term_lines = parse_payment_term_lines_json(&payment_term_json)
        .map_err(|reason| action(po, None, Some(reason)))?;
    let datetime = po
        .date
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).expect("midnight"))
        .and_utc();

    if dry_run {
        return Ok(Outcome::Created(action(po, None, None)));
    }

    let draft = create_draft_invoice(
        db,
        CreateDraftInput {
            number: Some(po.number.clone()),
            reference: Some(po.number.clone()),
            payment_reference: None,
            bank_account: None,
            datetime,
            delivery_date: None,
            customer_id: po.customer_id,
            payment_term_lines,
            header_tax_ids: Vec::new(),
            lines: pending,
        },
        timezone,
    )
    .await
    .map_err(|reason| action(po, None, Some(reason)))?;

    if let Err(reason) = link_site_invoice(db, site.id, draft.id).await {
        return Err(action(po, Some(draft.id), Some(reason)));
    }

    Ok(Outcome::Created(action(po, Some(draft.id), None)))
}

async fn summarize_purchase_order(
    db: &DatabaseConnection,
    po: &purchase_order::Model,
) -> Result<PurchaseOrderSummary, String> {
    let existing_invoice_id = existing_invoice_id(db, &po.number).await?;
    let lines = load_po_lines(db, po.id)
        .await?
        .into_iter()
        .map(|line| PurchaseOrderLineSummary {
            id: line.id,
            item_code: line.item_code.clone(),
            description: line.description,
            quantity: decimal_display(line.quantity),
            rate: decimal_display(line.rate),
        })
        .collect();
    Ok(PurchaseOrderSummary {
        id: po.id,
        number: po.number.clone(),
        date: po.date.to_string(),
        customer_id: po.customer_id,
        already_invoiced: existing_invoice_id.is_some(),
        existing_invoice_id,
        lines,
    })
}

async fn load_po_lines(
    db: &DatabaseConnection,
    purchase_order_id: i64,
) -> Result<Vec<purchase_order_line::Model>, String> {
    PurchaseOrderLineEntity::find()
        .filter(purchase_order_line::Column::PurchaseOrderId.eq(purchase_order_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())
}

async fn existing_invoice_id(db: &DatabaseConnection, number: &str) -> Result<Option<i64>, String> {
    if let Some(draft) = DraftInvoiceEntity::find()
        .filter(draft_invoice::Column::Number.eq(number))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(Some(draft.id));
    }
    if let Some(posted) = PostedInvoiceEntity::find()
        .filter(posted_invoice::Column::Number.eq(number))
        .one(db)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(Some(posted.draft_invoice_id));
    }
    Ok(None)
}

struct ProductIndex {
    by_reference: HashMap<String, i64>,
    by_name: HashMap<String, i64>,
}

impl ProductIndex {
    fn resolve(&self, item_code: &str) -> Option<i64> {
        let key = normalize_key(item_code);
        if key.is_empty() {
            return None;
        }
        self.by_reference
            .get(&key)
            .or_else(|| self.by_name.get(&key))
            .copied()
    }
}

async fn load_product_index(db: &DatabaseConnection) -> Result<ProductIndex, String> {
    let products = ProductEntity::find()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(index_products(products))
}

fn index_products(products: Vec<product::Model>) -> ProductIndex {
    let mut by_reference = HashMap::new();
    let mut by_name = HashMap::new();
    for product in products {
        if let Some(reference) = product
            .reference
            .as_deref()
            .map(normalize_key)
            .filter(|s| !s.is_empty())
        {
            by_reference.entry(reference).or_insert(product.id);
        }
        let name = normalize_key(&product.name);
        if !name.is_empty() {
            by_name.entry(name).or_insert(product.id);
        }
    }
    ProductIndex {
        by_reference,
        by_name,
    }
}

fn normalize_key(s: &str) -> String {
    s.trim().to_lowercase()
}

fn site_id_list(sites: &[site::Model]) -> String {
    sites
        .iter()
        .map(|s| format!("{} (#{})", s.name, s.id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn action(
    po: &purchase_order::Model,
    invoice_id: Option<i64>,
    reason: Option<String>,
) -> InvoiceAction {
    InvoiceAction {
        purchase_order_id: po.id,
        purchase_order_number: po.number.clone(),
        invoice_id,
        invoice_number: po.number.clone(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn product(id: i64, name: &str, reference: Option<&str>) -> product::Model {
        product::Model {
            id,
            created_at: None,
            updated_at: None,
            product_type: product::ProductType::Goods,
            reference: reference.map(str::to_string),
            remarks: None,
            name: name.into(),
            base_cost: Decimal::ZERO,
            sales_price: Decimal::ONE,
            hsn_code: 0,
        }
    }

    #[test]
    fn maps_item_code_to_product_reference() {
        let index = index_products(vec![
            product(1, "Gandola", Some("GND-1")),
            product(2, "TPI Panel", Some("TPI")),
        ]);
        assert_eq!(index.resolve("gnd-1"), Some(1));
        assert_eq!(index.resolve("TPI"), Some(2));
        assert_eq!(index.resolve("tpi panel"), Some(2));
        assert_eq!(index.resolve("missing"), None);
    }

    #[test]
    fn prefers_reference_over_name() {
        let index = index_products(vec![product(1, "Bolt", Some("A1")), product(2, "A1", None)]);
        assert_eq!(index.resolve("A1"), Some(1));
    }
}
