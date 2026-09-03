//! Shared site create/update/delete persistence used by HTTP handlers and Rune bindings.

use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::entities::{
    purchase_order::{self, Entity as PurchaseOrderEntity},
    site::{self, Entity as SiteEntity},
};
use crate::scope::{opt_string, sync_site_gandolas, sync_site_invoices, sync_site_purchase_orders};
use crate::site_status::SiteStatus;

#[derive(Debug, Clone)]
pub struct SiteFields {
    pub name: String,
    pub site_id: Option<String>,
    pub customer_id: i64,
    pub status: SiteStatus,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub address: Option<String>,
}

pub fn parse_optional_date(s: &str) -> Result<Option<NaiveDate>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    lariv_rs::datetime::parse_date(s)
        .map(Some)
        .ok_or_else(|| "invalid date".to_string())
}

pub fn parse_site_status(raw: &str) -> Result<Option<SiteStatus>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    SiteStatus::parse(trimmed)
        .map(Some)
        .ok_or_else(|| format!("invalid status {trimmed:?}"))
}

pub fn validate_site_fields(
    name: &str,
    customer_id: i64,
    status: SiteStatus,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    site_id: Option<String>,
    address: Option<String>,
) -> Result<SiteFields, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Name is required".into());
    }
    if customer_id <= 0 {
        return Err("Customer is required".into());
    }
    Ok(SiteFields {
        name: name.to_string(),
        site_id: site_id.and_then(opt_string),
        customer_id,
        status,
        start_date,
        end_date,
        address: address.and_then(opt_string),
    })
}

async fn sync_related(
    db: &DatabaseConnection,
    site_id: i64,
    customer_id: i64,
    gandolas: Option<&[i64]>,
    invoices: Option<&[i64]>,
    purchase_orders: Option<&[i64]>,
) -> Result<(), String> {
    if let Some(ids) = gandolas {
        sync_site_gandolas(db, site_id, ids).await?;
    }
    if let Some(ids) = invoices {
        sync_site_invoices(db, site_id, ids).await?;
    }
    if let Some(ids) = purchase_orders {
        sync_site_purchase_orders(db, site_id, customer_id, ids).await?;
    }
    Ok(())
}

pub async fn persist_new_site(
    db: &DatabaseConnection,
    fields: &SiteFields,
    gandolas: &[i64],
    invoices: &[i64],
    purchase_orders: &[i64],
) -> Result<site::Model, String> {
    let now = Utc::now();
    let model = site::ActiveModel {
        name: Set(fields.name.clone()),
        site_id: Set(fields.site_id.clone()),
        customer_id: Set(fields.customer_id),
        status: Set(fields.status),
        start_date: Set(fields.start_date),
        end_date: Set(fields.end_date),
        address: Set(fields.address.clone()),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let saved = model.insert(db).await.map_err(|e| e.to_string())?;
    sync_related(
        db,
        saved.id,
        saved.customer_id,
        Some(gandolas),
        Some(invoices),
        Some(purchase_orders),
    )
    .await?;
    Ok(saved)
}

pub async fn persist_updated_site(
    db: &DatabaseConnection,
    existing: &site::Model,
    fields: &SiteFields,
    gandolas: Option<&[i64]>,
    invoices: Option<&[i64]>,
    purchase_orders: Option<&[i64]>,
) -> Result<site::Model, String> {
    let now = Utc::now();
    let model = site::ActiveModel {
        id: Set(existing.id),
        name: Set(fields.name.clone()),
        site_id: Set(fields.site_id.clone()),
        customer_id: Set(fields.customer_id),
        status: Set(fields.status),
        start_date: Set(fields.start_date),
        end_date: Set(fields.end_date),
        address: Set(fields.address.clone()),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let saved = model.update(db).await.map_err(|e| e.to_string())?;
    sync_related(
        db,
        saved.id,
        saved.customer_id,
        gandolas,
        invoices,
        purchase_orders,
    )
    .await?;
    Ok(saved)
}

pub async fn delete_site(db: &DatabaseConnection, id: i64) -> Result<(), String> {
    if id <= 0 {
        return Err("delete_site requires a positive site id".into());
    }
    let existing = SiteEntity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("site {id} not found"))?;
    let linked_po = PurchaseOrderEntity::find()
        .filter(purchase_order::Column::SiteId.eq(existing.id))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    if linked_po.is_some() {
        return Err("cannot delete site while purchase orders still reference it".into());
    }
    SiteEntity::delete_by_id(existing.id)
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
