//! Site and PO PDF import helpers for the CLI.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};

use lariv_rs::plugins::customer::customer_type::CustomerType;
use lariv_rs::plugins::customer::entities::customer::{self, Entity as CustomerEntity};
use lariv_rs::plugins::filesystem::state::FilesystemState;

use crate::entities::gandola::{self, Entity as GandolaEntity};
use crate::entities::gandola_site_link::{self, Entity as GandolaSiteLinkEntity};
use crate::entities::preferences::Model as PreferencesModel;
use crate::entities::site::{self, Entity as SiteEntity};
use crate::po_from_pdf::{
    extract_purchase_order_from_pdf, form_from_extracted, store_purchase_order_pdf,
};
use crate::po_persist::{persist_new_purchase_order, purchase_order_number_taken};
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
    pub po_rent: String,
    pub po_dti: String,
    pub po_tpi: String,
    pub po_extn1: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PoImportReportEntry {
    pub file: String,
    pub po_number: Option<String>,
    pub site_id: Option<i64>,
    pub status: String,
    pub detail: String,
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

pub fn po_numbers_from_field(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

pub fn build_po_number_index(rows: &[SiteCsvRow]) -> HashMap<String, i64> {
    let mut index = HashMap::new();
    for row in rows {
        for number in po_numbers_for_csv_row(row) {
            index.insert(number, row.legacy_id);
        }
    }
    index
}

/// Maps PO numbers from the import CSV to Lariv `sites.id` rows already in the database.
pub async fn build_po_number_index_for_sites(
    db: &DatabaseConnection,
    rows: &[SiteCsvRow],
    customer_ids: &HashMap<i64, i64>,
) -> Result<HashMap<String, i64>, String> {
    let mut index = HashMap::new();
    for row in rows {
        let customer_id = customer_ids
            .get(&row.legacy_customer_id)
            .copied()
            .ok_or_else(|| format!("no Lariv customer for map id {}", row.legacy_customer_id))?;
        let site = find_existing_site(db, customer_id, &row.name)
            .await
            .ok_or_else(|| format!("site not in database: {}", row.name.trim()))?;
        for number in po_numbers_for_csv_row(row) {
            index.insert(number, site.id);
        }
    }
    Ok(index)
}

fn po_numbers_for_csv_row(row: &SiteCsvRow) -> Vec<String> {
    po_numbers_from_fields([&row.po_rent, &row.po_dti, &row.po_tpi, &row.po_extn1])
}

fn po_numbers_from_fields(fields: [&str; 4]) -> Vec<String> {
    let mut numbers = Vec::new();
    for field in fields {
        numbers.extend(po_numbers_from_field(field));
    }
    numbers
}

pub fn known_po_numbers(index: &HashMap<String, i64>) -> Vec<String> {
    let mut numbers: Vec<String> = index.keys().cloned().collect();
    numbers.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    numbers
}

pub fn match_po_number_in_filename(filename: &str, known_numbers: &[String]) -> Option<String> {
    let upper = filename.to_ascii_uppercase();
    for number in known_numbers {
        if upper.contains(&number.to_ascii_uppercase()) {
            return Some(number.clone());
        }
    }
    None
}

/// Result of importing a purchase-order PDF (Gemini extract + store + persist).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ImportPoPdfResult {
    pub id: i64,
    pub number: String,
    pub file_id: i64,
}

pub async fn import_po_pdf(
    db: &DatabaseConnection,
    fs: &FilesystemState,
    prefs: &PreferencesModel,
    site_id: i64,
    customer_id: i64,
    pdf_bytes: &[u8],
    filename: &str,
    tz: &str,
    dry_run: bool,
) -> Result<ImportPoPdfResult, String> {
    if dry_run {
        return Ok(ImportPoPdfResult {
            id: 0,
            number: String::new(),
            file_id: 0,
        });
    }
    let extracted =
        extract_purchase_order_from_pdf(&prefs.gemini_api_key, &prefs.gemini_model, pdf_bytes)
            .await?;
    let number = extracted.number.trim();
    if !number.is_empty() && purchase_order_number_taken(db, number, None).await {
        return Err(format!("purchase order {number} already exists"));
    }
    let file_id = store_purchase_order_pdf(fs, filename, pdf_bytes.to_vec()).await?;
    let form = form_from_extracted(&extracted, customer_id, site_id, file_id);
    let saved = persist_new_purchase_order(db, &form, tz).await?;
    Ok(ImportPoPdfResult {
        id: saved.id,
        number: saved.number,
        file_id,
    })
}

pub fn collect_pdf_paths(dir: &Path, recursive: bool) -> Result<Vec<std::path::PathBuf>, String> {
    let mut paths = Vec::new();
    if !dir.is_dir() {
        return Err(format!("not a directory: {}", dir.display()));
    }
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                paths.extend(collect_pdf_paths(&path, true)?);
            }
            continue;
        }
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

pub fn write_po_report(path: &Path, entries: &[PoImportReportEntry]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_row(legacy_id: i64, po_rent: &str, po_dti: &str, po_tpi: &str) -> SiteCsvRow {
        SiteCsvRow {
            legacy_id,
            legacy_customer_id: 21,
            name: format!("Site {legacy_id}"),
            status: "started".into(),
            start_date: String::new(),
            end_date: String::new(),
            address: String::new(),
            create_date: String::new(),
            write_date: String::new(),
            po_rent: po_rent.into(),
            po_dti: po_dti.into(),
            po_tpi: po_tpi.into(),
            po_extn1: String::new(),
        }
    }

    #[test]
    fn build_po_number_index_splits_comma_separated() {
        let rows = vec![
            sample_row(
                7,
                "P25RIN100982, P25RIN100984",
                "P25RIN100809",
                "P25RIN100983",
            ),
            sample_row(22, "WO/0038/25-26", "WO/0038/25-26", "WO/0038/25-26"),
        ];
        let index = build_po_number_index(&rows);
        assert_eq!(index.get("P25RIN100982"), Some(&7));
        assert_eq!(index.get("P25RIN100984"), Some(&7));
        assert_eq!(index.get("WO/0038/25-26"), Some(&22));
        assert_eq!(index.len(), 5);
    }

    #[test]
    fn match_po_number_prefers_longest_in_filename() {
        let numbers = vec![
            "P25RIN100982".into(),
            "P25RIN100984".into(),
            "P25RIN101616".into(),
        ];
        assert_eq!(
            match_po_number_in_filename("scan_P25RIN100984_final.pdf", &numbers),
            Some("P25RIN100984".into())
        );
        assert_eq!(
            match_po_number_in_filename("P25RIN101616.pdf", &numbers),
            Some("P25RIN101616".into())
        );
        assert_eq!(match_po_number_in_filename("no_match.pdf", &numbers), None);
    }

    #[test]
    fn po_numbers_from_field_trims_and_skips_empty() {
        let nums = po_numbers_from_field("  A, , B ,");
        assert_eq!(nums, vec!["A", "B"]);
    }
}
