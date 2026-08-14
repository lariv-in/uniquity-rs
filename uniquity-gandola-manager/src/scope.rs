use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Select, sea_query::Expr,
};

use lariv_rs::components::ManyToManyItem;
use lariv_rs::plugins::customer::entities::customer::Entity as CustomerEntity;
use lariv_rs::plugins::finance_invoices::entities::draft_invoice::{
    self, Entity as DraftInvoiceEntity,
};
use lariv_rs::plugins::finance_invoices::entities::posted_invoice::{
    self, Entity as PostedInvoiceEntity,
};
use lariv_rs::plugins::finance_invoices::logic::default_payment_term_lines_json;
use lariv_rs::plugins::finance_invoices::routes::{
    DraftInvoiceDetailRouteTag, PostedInvoiceDetailRouteTag,
};
use lariv_rs::plugins::finance_products::entities::product::Entity as ProductEntity;
use lariv_rs::plugins::users::state::AuthContext;

use crate::entities::{
    gandola::{self, Entity as GandolaEntity},
    gandola_site_link::{self, Entity as GandolaSiteLinkEntity},
    preferences::{self, Entity as PreferencesEntity},
    site::{self, Entity as SiteEntity},
    site_invoice_link::{self, Entity as SiteInvoiceLinkEntity},
};

pub fn is_superuser(auth: &AuthContext) -> bool {
    auth.user.is_superuser
}

pub fn scope_gandolas(query: Select<GandolaEntity>, auth: &AuthContext) -> Select<GandolaEntity> {
    if is_superuser(auth) {
        return query;
    }
    query.filter(Expr::cust("1 = 0"))
}

pub fn scope_sites(query: Select<SiteEntity>, auth: &AuthContext) -> Select<SiteEntity> {
    if is_superuser(auth) {
        return query;
    }
    query.filter(Expr::cust("1 = 0"))
}

pub fn apply_name_filter_gandolas(
    mut query: Select<GandolaEntity>,
    name: Option<&str>,
) -> Select<GandolaEntity> {
    if let Some(n) = name.filter(|s| !s.is_empty()) {
        query = query.filter(gandola::Column::Name.contains(n));
    }
    query
}

pub fn apply_name_filter_sites(
    mut query: Select<SiteEntity>,
    name: Option<&str>,
) -> Select<SiteEntity> {
    if let Some(n) = name.filter(|s| !s.is_empty()) {
        query = query.filter(site::Column::Name.contains(n));
    }
    query
}

pub async fn find_gandola_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<gandola::Model> {
    scope_gandolas(GandolaEntity::find_by_id(id), auth)
        .one(db)
        .await
        .ok()
        .flatten()
}

pub async fn find_site_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<site::Model> {
    scope_sites(SiteEntity::find_by_id(id), auth)
        .one(db)
        .await
        .ok()
        .flatten()
}

pub async fn load_sites_for_gandola(db: &DatabaseConnection, gandola_id: i64) -> Vec<site::Model> {
    GandolaEntity::find_by_id(gandola_id)
        .find_with_related(SiteEntity)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .flat_map(|(_, sites)| sites)
        .collect()
}

pub async fn load_gandolas_for_site(db: &DatabaseConnection, site_id: i64) -> Vec<gandola::Model> {
    SiteEntity::find_by_id(site_id)
        .find_with_related(GandolaEntity)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .flat_map(|(_, gandolas)| gandolas)
        .collect()
}

pub async fn site_items_for_gandola(
    db: &DatabaseConnection,
    gandola_id: i64,
) -> Vec<ManyToManyItem> {
    load_sites_for_gandola(db, gandola_id)
        .await
        .into_iter()
        .map(|s| ManyToManyItem::new(s.id.to_string(), s.name))
        .collect()
}

pub async fn gandola_items_for_site(db: &DatabaseConnection, site_id: i64) -> Vec<ManyToManyItem> {
    load_gandolas_for_site(db, site_id)
        .await
        .into_iter()
        .map(|g| ManyToManyItem::new(g.id.to_string(), g.name))
        .collect()
}

pub async fn site_items_from_ids(db: &DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let models = SiteEntity::find()
        .filter(site::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            models
                .iter()
                .find(|s| s.id == *id)
                .map(|s| ManyToManyItem::new(s.id.to_string(), s.name.clone()))
        })
        .collect()
}

pub async fn gandola_items_from_ids(db: &DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let models = GandolaEntity::find()
        .filter(gandola::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            models
                .iter()
                .find(|g| g.id == *id)
                .map(|g| ManyToManyItem::new(g.id.to_string(), g.name.clone()))
        })
        .collect()
}

pub async fn sync_gandola_sites<C: ConnectionTrait>(
    db: &C,
    gandola_id: i64,
    site_ids: &[i64],
) -> Result<(), String> {
    GandolaSiteLinkEntity::delete_many()
        .filter(gandola_site_link::Column::GandolaId.eq(gandola_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    let mut seen = std::collections::BTreeSet::new();
    for &site_id in site_ids {
        if site_id <= 0 || !seen.insert(site_id) {
            continue;
        }
        gandola_site_link::ActiveModel {
            gandola_id: Set(gandola_id),
            site_id: Set(site_id),
        }
        .insert(db)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn sync_site_gandolas<C: ConnectionTrait>(
    db: &C,
    site_id: i64,
    gandola_ids: &[i64],
) -> Result<(), String> {
    GandolaSiteLinkEntity::delete_many()
        .filter(gandola_site_link::Column::SiteId.eq(site_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    let mut seen = std::collections::BTreeSet::new();
    for &gandola_id in gandola_ids {
        if gandola_id <= 0 || !seen.insert(gandola_id) {
            continue;
        }
        gandola_site_link::ActiveModel {
            gandola_id: Set(gandola_id),
            site_id: Set(site_id),
        }
        .insert(db)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn load_invoices_for_site(
    db: &DatabaseConnection,
    site_id: i64,
) -> Vec<draft_invoice::Model> {
    SiteEntity::find_by_id(site_id)
        .find_with_related(DraftInvoiceEntity)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .flat_map(|(_, invoices)| invoices)
        .collect()
}

pub async fn load_sites_for_invoice(
    db: &DatabaseConnection,
    draft_invoice_id: i64,
) -> Vec<site::Model> {
    SiteInvoiceLinkEntity::find()
        .filter(site_invoice_link::Column::DraftInvoiceId.eq(draft_invoice_id))
        .find_also_related(SiteEntity)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(_, site)| site)
        .collect()
}

fn invoice_label(id: i64, number: &Option<String>) -> String {
    match number.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(n) => n.to_string(),
        None => format!("#{id}"),
    }
}

async fn invoice_href(db: &DatabaseConnection, draft_id: i64) -> String {
    if let Ok(Some(posted)) = PostedInvoiceEntity::find()
        .filter(posted_invoice::Column::DraftInvoiceId.eq(draft_id))
        .one(db)
        .await
    {
        return PostedInvoiceDetailRouteTag::new(posted.id).url();
    }
    DraftInvoiceDetailRouteTag::new(draft_id).url()
}

pub async fn invoice_items_for_site(db: &DatabaseConnection, site_id: i64) -> Vec<ManyToManyItem> {
    load_invoices_for_site(db, site_id)
        .await
        .into_iter()
        .map(|d| ManyToManyItem::new(d.id.to_string(), invoice_label(d.id, &d.number)))
        .collect()
}

pub async fn site_items_for_invoice(
    db: &DatabaseConnection,
    draft_invoice_id: i64,
) -> Vec<ManyToManyItem> {
    load_sites_for_invoice(db, draft_invoice_id)
        .await
        .into_iter()
        .map(|s| ManyToManyItem::new(s.id.to_string(), s.name))
        .collect()
}

pub async fn related_invoices_for_site(
    db: &DatabaseConnection,
    site_id: i64,
) -> Vec<(i64, String, String)> {
    let drafts = load_invoices_for_site(db, site_id).await;
    let mut out = Vec::with_capacity(drafts.len());
    for d in drafts {
        let href = invoice_href(db, d.id).await;
        out.push((d.id, invoice_label(d.id, &d.number), href));
    }
    out
}

pub async fn invoice_items_from_ids(db: &DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let models = DraftInvoiceEntity::find()
        .filter(draft_invoice::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            models
                .iter()
                .find(|d| d.id == *id)
                .map(|d| ManyToManyItem::new(d.id.to_string(), invoice_label(d.id, &d.number)))
        })
        .collect()
}

pub async fn sync_site_invoices<C: ConnectionTrait>(
    db: &C,
    site_id: i64,
    draft_invoice_ids: &[i64],
) -> Result<(), String> {
    SiteInvoiceLinkEntity::delete_many()
        .filter(site_invoice_link::Column::SiteId.eq(site_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    let mut seen = std::collections::BTreeSet::new();
    for &draft_invoice_id in draft_invoice_ids {
        if draft_invoice_id <= 0 || !seen.insert(draft_invoice_id) {
            continue;
        }
        site_invoice_link::ActiveModel {
            site_id: Set(site_id),
            draft_invoice_id: Set(draft_invoice_id),
        }
        .insert(db)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn sync_invoice_sites<C: ConnectionTrait>(
    db: &C,
    draft_invoice_id: i64,
    site_ids: &[i64],
) -> Result<(), String> {
    SiteInvoiceLinkEntity::delete_many()
        .filter(site_invoice_link::Column::DraftInvoiceId.eq(draft_invoice_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    let mut seen = std::collections::BTreeSet::new();
    for &site_id in site_ids {
        if site_id <= 0 || !seen.insert(site_id) {
            continue;
        }
        site_invoice_link::ActiveModel {
            site_id: Set(site_id),
            draft_invoice_id: Set(draft_invoice_id),
        }
        .insert(db)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn customer_name(db: &DatabaseConnection, customer_id: i64) -> String {
    if customer_id <= 0 {
        return String::new();
    }
    CustomerEntity::find_by_id(customer_id)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|c| c.name)
        .unwrap_or_else(|| format!("#{customer_id}"))
}

pub async fn product_name(db: &DatabaseConnection, product_id: Option<i64>) -> String {
    let Some(id) = product_id.filter(|&id| id > 0) else {
        return String::new();
    };
    ProductEntity::find_by_id(id)
        .one(db)
        .await
        .ok()
        .flatten()
        .map(|p| p.name)
        .unwrap_or_else(|| format!("#{id}"))
}

pub fn parse_optional_i64(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        None
    } else {
        s.parse().ok().filter(|&id| id > 0)
    }
}

pub async fn load_preferences(db: &DatabaseConnection) -> preferences::Model {
    if let Ok(Some(p)) = PreferencesEntity::find_by_id(1i64).one(db).await {
        return p;
    }
    let now = Utc::now();
    let am = preferences::ActiveModel {
        id: Set(1),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        gandola_product_id: Set(None),
        tpi_product_id: Set(None),
        dti_product_id: Set(None),
        payment_term_lines_json: Set(Some(default_payment_term_lines_json())),
    };
    am.insert(db).await.unwrap_or(preferences::Model {
        id: 1,
        created_at: Some(now),
        updated_at: Some(now),
        gandola_product_id: None,
        tpi_product_id: None,
        dti_product_id: None,
        payment_term_lines_json: Some(default_payment_term_lines_json()),
    })
}

pub fn opt_string(s: String) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
