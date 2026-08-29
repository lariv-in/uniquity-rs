//! Site import helpers for the CLI.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::Deserialize;

use lariv_rs::plugins::customer::customer_type::CustomerType;
use lariv_rs::plugins::customer::entities::customer::{self, Entity as CustomerEntity};

use crate::entities::gandola::{self, Entity as GandolaEntity};
use crate::entities::gandola_site_link::{self, Entity as GandolaSiteLinkEntity};
use crate::entities::site::{self, Entity as SiteEntity};
use crate::scope::opt_string;
use crate::site_status::SiteStatus;

#[derive(Debug, Clone, Deserialize)]
pub struct CustomerMapRow {
    pub legacy_customer_id: i64,
    pub customer_name: String,
    pub gstin: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GandolaCsvRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GandolaSiteRelRow {
    #[serde(alias = "gandola_manager_gandola_id")]
    pub gandola_id: i64,
    #[serde(alias = "gandola_manager_site_id")]
    pub site_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteCsvRow {
    pub legacy_id: i64,
    pub legacy_customer_id: i64,
    pub name: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub address: String,
    pub create_date: String,
    pub write_date: String,
}

pub fn load_customer_map_csv(path: &Path) -> Result<Vec<CustomerMapRow>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    reader
        .deserialize()
        .collect::<Result<Vec<CustomerMapRow>, _>>()
        .map_err(|e| e.to_string())
}

pub fn load_sites_csv(path: &Path) -> Result<Vec<SiteCsvRow>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    reader
        .deserialize()
        .collect::<Result<Vec<SiteCsvRow>, _>>()
        .map_err(|e| e.to_string())
}

pub fn load_gandolas_csv(path: &Path) -> Result<Vec<GandolaCsvRow>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    reader
        .deserialize()
        .collect::<Result<Vec<GandolaCsvRow>, _>>()
        .map_err(|e| e.to_string())
}

pub fn load_gandola_sites_csv(path: &Path) -> Result<Vec<GandolaSiteRelRow>, String> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    reader
        .deserialize()
        .collect::<Result<Vec<GandolaSiteRelRow>, _>>()
        .map_err(|e| e.to_string())
}

async fn find_customer_by_gstin(db: &DatabaseConnection, gstin: &str) -> Option<customer::Model> {
    let gstin = gstin.trim();
    if gstin.is_empty() {
        return None;
    }
    if let Ok(Some(c)) = CustomerEntity::find()
        .filter(customer::Column::Gstin.eq(gstin))
        .one(db)
        .await
    {
        return Some(c);
    }
    let upper = gstin.to_ascii_uppercase();
    if upper != gstin {
        if let Ok(Some(c)) = CustomerEntity::find()
            .filter(customer::Column::Gstin.eq(upper))
            .one(db)
            .await
        {
            return Some(c);
        }
    }
    None
}

async fn find_customer_by_name(db: &DatabaseConnection, name: &str) -> Option<customer::Model> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    if let Ok(Some(c)) = CustomerEntity::find()
        .filter(customer::Column::Name.eq(name))
        .one(db)
        .await
    {
        return Some(c);
    }
    let Ok(matches) = CustomerEntity::find()
        .filter(customer::Column::Name.contains(name))
        .all(db)
        .await
    else {
        return None;
    };
    let exact: Vec<_> = matches
        .iter()
        .filter(|c| c.name.trim().eq_ignore_ascii_case(name))
        .cloned()
        .collect();
    if exact.len() == 1 {
        return exact.into_iter().next();
    }
    None
}

pub async fn create_customer(
    db: &DatabaseConnection,
    name: &str,
    gstin: &str,
) -> Result<customer::Model, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("customer name is required".into());
    }
    let now = Utc::now();
    let gstin = gstin.trim();
    let model = customer::ActiveModel {
        customer_type: Set(CustomerType::Business),
        name: Set(name.to_string()),
        gstin: Set(opt_string(gstin.to_string())),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    model.insert(db).await.map_err(|e| e.to_string())
}

pub async fn resolve_customer(
    db: &DatabaseConnection,
    name: &str,
    gstin: &str,
    create_missing: bool,
    dry_run: bool,
) -> Result<customer::Model, String> {
    if let Some(c) = find_customer_by_gstin(db, gstin).await {
        return Ok(c);
    }
    if let Some(c) = find_customer_by_name(db, name).await {
        return Ok(c);
    }
    if !create_missing {
        return Err(format!(
            "customer not found for name={name:?} gstin={gstin:?}"
        ));
    }
    if dry_run {
        return Err(format!(
            "dry-run: would create customer name={name:?} gstin={gstin:?}"
        ));
    }
    create_customer(db, name, gstin).await
}

pub async fn resolve_customer_ids(
    db: &DatabaseConnection,
    rows: &[CustomerMapRow],
    create_missing: bool,
    dry_run: bool,
) -> Result<HashMap<i64, i64>, String> {
    let mut map = HashMap::new();
    for row in rows {
        let id = resolve_or_create_customer_logged(db, row, create_missing, dry_run).await?;
        if let Some(id) = id {
            map.insert(row.legacy_customer_id, id);
        }
    }
    Ok(map)
}

async fn resolve_or_create_customer_logged(
    db: &DatabaseConnection,
    row: &CustomerMapRow,
    create_missing: bool,
    dry_run: bool,
) -> Result<Option<i64>, String> {
    if let Some(c) = find_customer_by_gstin(db, &row.gstin).await {
        return Ok(Some(c.id));
    }
    if let Some(c) = find_customer_by_name(db, &row.customer_name).await {
        return Ok(Some(c.id));
    }
    if !create_missing {
        return Err(format!(
            "customer not found for name={:?} gstin={:?}",
            row.customer_name, row.gstin
        ));
    }
    if dry_run {
        eprintln!(
            "dry-run: would create customer map_id={} name={:?} gstin={:?}",
            row.legacy_customer_id, row.customer_name, row.gstin
        );
        return Ok(None);
    }
    let created = create_customer(db, &row.customer_name, &row.gstin).await?;
    eprintln!(
        "created customer id={} map_id={} name={}",
        created.id, row.legacy_customer_id, created.name
    );
    Ok(Some(created.id))
}

pub async fn resolve_import_gandola_id(
    db: &DatabaseConnection,
    gandola_id: Option<i64>,
) -> Result<i64, String> {
    if let Some(id) = gandola_id {
        if id <= 0 {
            return Err("gandola id must be positive".into());
        }
        if GandolaEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .is_none()
        {
            return Err(format!("gandola id {id} not found"));
        }
        return Ok(id);
    }
    let gandolas = GandolaEntity::find()
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    match gandolas.len() {
        0 => Err("no gandolas in database; create one in the UI or pass --gandola-id".into()),
        1 => Ok(gandolas[0].id),
        n => {
            let listing = gandolas
                .iter()
                .map(|g| format!("id={} name={}", g.id, g.name))
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!("{n} gandolas found; pass --gandola-id ({listing})"))
        }
    }
}

async fn ensure_site_gandola_link(
    db: &DatabaseConnection,
    site_id: i64,
    gandola_id: i64,
) -> Result<(), String> {
    if site_id <= 0 {
        return Ok(());
    }
    let exists = GandolaSiteLinkEntity::find()
        .filter(gandola_site_link::Column::SiteId.eq(site_id))
        .filter(gandola_site_link::Column::GandolaId.eq(gandola_id))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    if exists.is_some() {
        return Ok(());
    }
    gandola_site_link::ActiveModel {
        gandola_id: Set(gandola_id),
        site_id: Set(site_id),
    }
    .insert(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn ensure_site_gandola_link_for_import(
    db: &DatabaseConnection,
    site_id: i64,
    gandola_id: i64,
) -> Result<(), String> {
    ensure_site_gandola_link(db, site_id, gandola_id).await
}

async fn find_gandola_by_name(db: &DatabaseConnection, name: &str) -> Option<gandola::Model> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    lariv_rs::web::opt_or_log(
        GandolaEntity::find()
            .filter(gandola::Column::Name.eq(name))
            .one(db)
            .await,
        "db find one",
    )
}

pub async fn import_gandola_row(
    db: &DatabaseConnection,
    name: &str,
    dry_run: bool,
) -> Result<i64, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("gandola name is required".into());
    }
    if let Some(existing) = find_gandola_by_name(db, name).await {
        return Ok(existing.id);
    }
    if dry_run {
        return Ok(0);
    }
    let now = Utc::now();
    let model = gandola::ActiveModel {
        name: Set(name.to_string()),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    model
        .insert(db)
        .await
        .map(|m| m.id)
        .map_err(|e| e.to_string())
}

/// Source Odoo gandola id → Lariv `gandolas.id`.
pub async fn import_gandolas_from_csv(
    db: &DatabaseConnection,
    path: &Path,
    dry_run: bool,
) -> Result<HashMap<i64, i64>, String> {
    let rows = load_gandolas_csv(path)?;
    let mut map = HashMap::new();
    for row in rows {
        let lariv_id = import_gandola_row(db, &row.name, dry_run).await?;
        if lariv_id > 0 {
            map.insert(row.id, lariv_id);
            println!(
                "gandola source id={} -> id={} name={}",
                row.id, lariv_id, row.name
            );
        } else if dry_run {
            println!("dry-run ok gandola source id={} name={}", row.id, row.name);
        }
    }
    Ok(map)
}

async fn lariv_gandola_id_for_source(
    db: &DatabaseConnection,
    source_id: i64,
    gandola_map: &HashMap<i64, i64>,
) -> Result<i64, String> {
    if let Some(id) = gandola_map.get(&source_id).copied() {
        return Ok(id);
    }
    if GandolaEntity::find_by_id(source_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(source_id);
    }
    Err(format!("gandola source id {source_id} not found"))
}

/// Applies Odoo `gandola_manager_gandola_gandola_manager_site_rel` rows to `gandola_sites`.
pub async fn apply_gandola_site_links(
    db: &DatabaseConnection,
    path: &Path,
    gandola_map: &HashMap<i64, i64>,
    site_map: &HashMap<i64, i64>,
    dry_run: bool,
) -> Result<usize, String> {
    let rows = load_gandola_sites_csv(path)?;
    let mut linked = 0usize;
    for row in rows {
        let lariv_gandola = lariv_gandola_id_for_source(db, row.gandola_id, gandola_map).await?;
        let lariv_site = match site_map.get(&row.site_id).copied() {
            Some(id) => id,
            None => {
                eprintln!(
                    "skip link: site source id {} not imported (missing from sites.csv?)",
                    row.site_id
                );
                continue;
            }
        };
        if dry_run {
            println!(
                "dry-run link gandola {} -> site {} (source gandola={} site={})",
                lariv_gandola, lariv_site, row.gandola_id, row.site_id
            );
        } else {
            ensure_site_gandola_link(db, lariv_site, lariv_gandola).await?;
            println!(
                "linked gandola id={} site id={} (source gandola={} site={})",
                lariv_gandola, lariv_site, row.gandola_id, row.site_id
            );
        }
        linked += 1;
    }
    Ok(linked)
}

fn parse_optional_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    lariv_rs::datetime::parse_date(s)
}

fn parse_timestamp(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
}

pub async fn find_existing_site(
    db: &DatabaseConnection,
    customer_id: i64,
    name: &str,
) -> Option<site::Model> {
    lariv_rs::web::opt_or_log(
        SiteEntity::find()
            .filter(site::Column::CustomerId.eq(customer_id))
            .filter(site::Column::Name.eq(name.trim()))
            .one(db)
            .await,
        "db find one",
    )
}

pub async fn import_site_row(
    db: &DatabaseConnection,
    row: &SiteCsvRow,
    customer_id: i64,
    dry_run: bool,
) -> Result<i64, String> {
    let name = row.name.trim();
    if name.is_empty() {
        return Err("site name is required".into());
    }

    if let Some(existing) = find_existing_site(db, customer_id, name).await {
        return Ok(existing.id);
    }
    if dry_run {
        return Ok(0);
    }
    let status = SiteStatus::parse(row.status.trim()).unwrap_or_default();
    let created_at = parse_timestamp(&row.create_date);
    let updated_at = parse_timestamp(&row.write_date);
    let model = site::ActiveModel {
        name: Set(name.to_string()),
        customer_id: Set(customer_id),
        status: Set(status),
        start_date: Set(parse_optional_date(&row.start_date)),
        end_date: Set(parse_optional_date(&row.end_date)),
        address: Set(opt_string(row.address.clone())),
        created_at: Set(created_at.or_else(|| Some(Utc::now()))),
        updated_at: Set(updated_at.or_else(|| Some(Utc::now()))),
        ..Default::default()
    };
    model
        .insert(db)
        .await
        .map(|m| m.id)
        .map_err(|e| e.to_string())
}
