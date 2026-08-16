//! Shared import command handlers for CLI and tests.

use std::collections::HashMap;
use std::path::PathBuf;

use sea_orm::{DatabaseConnection, EntityTrait};

use lariv_rs::plugins::filesystem::state::FilesystemState;

use crate::entities::site::Entity as SiteEntity;
use crate::import::{
    apply_gandola_site_links, build_po_number_index_for_sites, collect_pdf_paths,
    ensure_site_gandola_link_for_import, find_existing_site, import_gandolas_from_csv,
    import_po_pdf, import_site_row, known_po_numbers, load_customer_map_csv, load_sites_csv,
    match_po_number_in_filename, resolve_customer_ids, resolve_import_gandola_id,
    write_po_report, PoImportReportEntry,
};
use crate::scope::{customer_name, load_preferences};

const DEFAULT_TZ: &str = "Asia/Kolkata";

pub async fn run_import_sites(
    db: &DatabaseConnection,
    sites_path: &PathBuf,
    customers_path: &PathBuf,
    gandolas_path: &PathBuf,
    gandola_sites_path: &PathBuf,
    fallback_gandola_id: Option<i64>,
    dry_run: bool,
    create_missing_customers: bool,
) -> anyhow::Result<()> {
    let customer_rows = load_customer_map_csv(customers_path)
        .map_err(|e| anyhow::anyhow!("load customers: {e}"))?;
    let site_rows = load_sites_csv(sites_path).map_err(|e| anyhow::anyhow!("load sites: {e}"))?;

    let gandola_map = if gandolas_path.is_file() {
        import_gandolas_from_csv(db, gandolas_path, dry_run)
            .await
            .map_err(|e| anyhow::anyhow!("import gandolas: {e}"))?
    } else {
        HashMap::new()
    };

    let customer_ids = resolve_customer_ids(
        db,
        &customer_rows,
        create_missing_customers,
        dry_run,
    )
    .await
    .map_err(|e| anyhow::anyhow!("resolve customers: {e}"))?;

    let mut site_map = HashMap::new();
    for row in &site_rows {
        let lariv_customer_id = match customer_ids.get(&row.legacy_customer_id).copied() {
            Some(id) => id,
            None if dry_run => {
                eprintln!(
                    "dry-run skip site name={}: customer map id {} not resolved",
                    row.name, row.legacy_customer_id
                );
                continue;
            }
            None => {
                return Err(anyhow::anyhow!(
                    "no Lariv customer for map id {}",
                    row.legacy_customer_id
                ));
            }
        };

        let existing_before = find_existing_site(db, lariv_customer_id, &row.name).await;
        let site_id = import_site_row(db, row, lariv_customer_id, dry_run)
            .await
            .map_err(|e| anyhow::anyhow!("site {}: {e}", row.name))?;

        if site_id > 0 {
            site_map.insert(row.legacy_id, site_id);
        }

        if existing_before.is_some() {
            eprintln!(
                "warn: site already exists name={} -> id={}",
                row.name, site_id
            );
        }

        if dry_run {
            println!(
                "dry-run ok source site id={} name={} customer_id={} customer={}",
                row.legacy_id,
                row.name,
                lariv_customer_id,
                customer_name(db, lariv_customer_id).await
            );
        } else if site_id > 0 {
            println!(
                "imported source site id={} -> id={} name={} status={}",
                row.legacy_id, site_id, row.name, row.status
            );
        }
    }

    if gandola_sites_path.is_file() {
        let linked = apply_gandola_site_links(db, gandola_sites_path, &gandola_map, &site_map, dry_run)
            .await
            .map_err(|e| anyhow::anyhow!("gandola-site links: {e}"))?;
        println!("applied {linked} gandola-site links");
    } else if !site_map.is_empty() {
        let gandola_id = resolve_import_gandola_id(db, fallback_gandola_id)
            .await
            .map_err(|e| anyhow::anyhow!("resolve gandola: {e}"))?;
        if !dry_run {
            println!("linking all imported sites to gandola id={gandola_id}");
        }
        for (&source_site_id, &lariv_site_id) in &site_map {
            if dry_run {
                println!(
                    "dry-run link gandola {gandola_id} -> site {lariv_site_id} (source site={source_site_id})"
                );
            } else {
                ensure_site_gandola_link_for_import(db, lariv_site_id, gandola_id)
                    .await
                    .map_err(|e| anyhow::anyhow!("link site {lariv_site_id}: {e}"))?;
            }
        }
    }

    Ok(())
}

pub async fn run_import_po_pdfs(
    db: &DatabaseConnection,
    fs: &FilesystemState,
    sites_path: &PathBuf,
    customers_path: &PathBuf,
    pdf_dir: &PathBuf,
    out_path: &PathBuf,
    recursive: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    let customer_rows = load_customer_map_csv(customers_path)
        .map_err(|e| anyhow::anyhow!("load customers: {e}"))?;
    let site_rows = load_sites_csv(sites_path).map_err(|e| anyhow::anyhow!("load sites: {e}"))?;

    let customer_ids = resolve_customer_ids(db, &customer_rows, false, dry_run)
        .await
        .map_err(|e| anyhow::anyhow!("resolve customers: {e}"))?;

    let po_index = build_po_number_index_for_sites(db, &site_rows, &customer_ids)
        .await
        .map_err(|e| anyhow::anyhow!("build PO index: {e}"))?;
    let known = known_po_numbers(&po_index);
    let prefs = load_preferences(db).await;

    if !dry_run && prefs.gemini_api_key.trim().is_empty() {
        anyhow::bail!("Gemini API key not set in Gandola preferences");
    }

    let pdf_paths = collect_pdf_paths(pdf_dir, recursive)
        .map_err(|e| anyhow::anyhow!("collect PDFs: {e}"))?;

    let mut report = Vec::new();
    for path in pdf_paths {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.pdf");
        let po_number = match_po_number_in_filename(filename, &known);
        let site_id = po_number.as_ref().and_then(|n| po_index.get(n).copied());

        let entry = if po_number.is_none() {
            PoImportReportEntry {
                file: path.display().to_string(),
                po_number: None,
                site_id: None,
                status: "unmatched".into(),
                detail: "no known PO number in filename".into(),
            }
        } else if site_id.is_none() {
            PoImportReportEntry {
                file: path.display().to_string(),
                po_number,
                site_id: None,
                status: "unmapped".into(),
                detail: "PO matched but site not found in database".into(),
            }
        } else {
            let site_id = site_id.unwrap();
            let site = SiteEntity::find_by_id(site_id)
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("site id {site_id} not found"))?;

            let bytes = std::fs::read(&path)?;
            match import_po_pdf(
                db,
                fs,
                &prefs,
                site.id,
                site.customer_id,
                &bytes,
                filename,
                DEFAULT_TZ,
                dry_run,
            )
            .await
            {
                Ok(number) => PoImportReportEntry {
                    file: path.display().to_string(),
                    po_number,
                    site_id: Some(site_id),
                    status: if dry_run { "dry-run" } else { "ok" }.into(),
                    detail: if dry_run {
                        "would import".into()
                    } else {
                        format!("created PO {number}")
                    },
                },
                Err(e) => PoImportReportEntry {
                    file: path.display().to_string(),
                    po_number,
                    site_id: Some(site_id),
                    status: "error".into(),
                    detail: e,
                },
            }
        };

        println!(
            "{} {} {:?} site_id={:?} {}",
            entry.status,
            entry.file,
            entry.po_number,
            entry.site_id,
            entry.detail
        );
        report.push(entry);
    }

    if !dry_run {
        write_po_report(out_path, &report).map_err(|e| anyhow::anyhow!("write report: {e}"))?;
        println!("wrote {}", out_path.display());
    }

    Ok(())
}
