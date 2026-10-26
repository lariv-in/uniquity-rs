use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend,
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select, Statement, sea_query::Expr,
};

use lariv_core::components::ManyToManyItem;
use lariv_plugin_contacts::entities::CompanyEntity;
use lariv_plugin_filesystem::entities::filesystem_node::Entity as VNodeEntity;
use lariv_plugin_filesystem::routes::VNodeFileSelectInRouteTag;
use lariv_plugin_filesystem::storage::{DynFilestore, UnimplementedFilestore};
use lariv_plugin_finance_accounts::scope::{
    CurrencyFormat, load_default_currency_format, load_journal_currency_format,
};
use lariv_plugin_finance_invoices::entities::cancelled_invoice::{
    self, Entity as CancelledInvoiceEntity,
};
use lariv_plugin_finance_invoices::entities::draft_invoice::{self, Entity as DraftInvoiceEntity};
use lariv_plugin_finance_invoices::entities::draft_invoice_goods_line::{
    self, Entity as DraftInvoiceGoodsLineEntity,
};
use lariv_plugin_finance_invoices::entities::paid_invoice::{self, Entity as PaidInvoiceEntity};
use lariv_plugin_finance_invoices::entities::partially_paid_invoice::{
    self, Entity as PartiallyPaidInvoiceEntity,
};
use lariv_plugin_finance_invoices::entities::payment::Entity as PaymentEntity;
use lariv_plugin_finance_invoices::entities::posted_invoice::{
    self, Entity as PostedInvoiceEntity,
};
use lariv_plugin_finance_invoices::entities::posted_invoice_goods_line::{
    self, Entity as PostedInvoiceGoodsLineEntity,
};
use lariv_plugin_finance_invoices::logic::bill_to::{PartyLabels, load_bill_to};
use lariv_plugin_finance_invoices::logic::{
    InvoiceDateFormats, InvoiceListMetrics, cancelled_invoice_list_metrics,
    default_payment_term_lines_json, draft_invoice_list_metrics, load_invoice_date_formats,
    posted_invoice_list_metrics, posted_invoice_open_balance,
};
use lariv_plugin_finance_invoices::routes::{
    CancelledInvoiceDetailRouteTag, DraftInvoiceDetailRouteTag, PaidInvoiceDetailRouteTag,
    PartiallyPaidInvoiceDetailRouteTag, PostedInvoiceDetailRouteTag,
};
use lariv_plugin_finance_products::entities::product::Entity as ProductEntity;
use lariv_plugin_users::state::AuthContext;

use crate::entities::{
    gandola::{self, Entity as GandolaEntity},
    gandola_site_link::{self, Entity as GandolaSiteLinkEntity},
    preferences::{self, Entity as PreferencesEntity},
    purchase_order::{self, Entity as PurchaseOrderEntity},
    site::{self, Entity as SiteEntity},
    site_invoice_link::{self, Entity as SiteInvoiceLinkEntity},
};

pub fn is_superuser(auth: &AuthContext) -> bool {
    lariv_plugin_users::roles::Superuser::matches(&auth.role)
}

/// Gandola Manager allowlist (`hr`, plus any superuser): list, detail, create, edit, and delete.
pub fn can_manage(auth: &AuthContext) -> bool {
    lariv_plugin_users::role_authorization::principal_allowed(
        auth,
        &lariv_plugin_users::role_authorization::roles_for::<crate::routes::GandolaAccess>(),
    )
}

pub fn scope_gandolas(query: Select<GandolaEntity>, auth: &AuthContext) -> Select<GandolaEntity> {
    if can_manage(auth) {
        return query;
    }
    query.filter(Expr::cust("1 = 0"))
}

pub fn scope_sites(query: Select<SiteEntity>, auth: &AuthContext) -> Select<SiteEntity> {
    if can_manage(auth) {
        return query;
    }
    query.filter(Expr::cust("1 = 0"))
}

pub fn scope_purchase_orders(
    query: Select<PurchaseOrderEntity>,
    auth: &AuthContext,
) -> Select<PurchaseOrderEntity> {
    if can_manage(auth) {
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

pub fn apply_site_id_filter_sites(
    mut query: Select<SiteEntity>,
    site_id: Option<&str>,
) -> Select<SiteEntity> {
    if let Some(s) = site_id.filter(|s| !s.is_empty()) {
        query = query.filter(site::Column::SiteId.contains(s));
    }
    query
}

pub fn apply_number_filter_purchase_orders(
    mut query: Select<PurchaseOrderEntity>,
    number: Option<&str>,
) -> Select<PurchaseOrderEntity> {
    if let Some(n) = number.filter(|s| !s.is_empty()) {
        query = query.filter(purchase_order::Column::Number.contains(n));
    }
    query
}

pub async fn find_gandola_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<gandola::Model> {
    lariv_core::web::opt_or_log(
        scope_gandolas(GandolaEntity::find_by_id(id), auth)
            .one(db)
            .await,
        "find by id",
    )
}

pub async fn find_site_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<site::Model> {
    lariv_core::web::opt_or_log(
        scope_sites(SiteEntity::find_by_id(id), auth).one(db).await,
        "find by id",
    )
}

pub async fn find_purchase_order_scoped(
    db: &DatabaseConnection,
    id: i64,
    auth: &AuthContext,
) -> Option<purchase_order::Model> {
    lariv_core::web::opt_or_log(
        scope_purchase_orders(PurchaseOrderEntity::find_by_id(id), auth)
            .one(db)
            .await,
        "find by id",
    )
}

pub async fn vnode_name(db: &DatabaseConnection, vnode_id: Option<i64>) -> String {
    let Some(id) = vnode_id.filter(|&id| id > 0) else {
        return String::new();
    };
    lariv_core::web::opt_or_log(VNodeEntity::find_by_id(id).one(db).await, "find by id")
        .map(|n| n.name)
        .unwrap_or_else(|| format!("#{id}"))
}

pub async fn vnode_path(db: &DatabaseConnection, vnode_id: Option<i64>) -> String {
    let Some(id) = vnode_id.filter(|&id| id > 0) else {
        return String::new();
    };
    let Some(node) =
        lariv_core::web::opt_or_log(VNodeEntity::find_by_id(id).one(db).await, "find by id")
    else {
        return format!("#{id}");
    };
    lariv_plugin_filesystem::node::get_path(db, &node).await
}

/// Accepts an empty id, or the id of a filesystem directory.
pub async fn require_vnode_directory(
    db: &DatabaseConnection,
    vnode_id: Option<i64>,
) -> Result<Option<i64>, String> {
    let Some(id) = vnode_id.filter(|&id| id > 0) else {
        return Ok(None);
    };
    match VNodeEntity::find_by_id(id).one(db).await {
        Ok(Some(node)) if node.is_directory => Ok(Some(node.id)),
        Ok(Some(_)) => Err("Purchase order files directory must be a folder.".into()),
        Ok(None) => Err("Purchase order files directory was not found.".into()),
        Err(e) => {
            tracing::error!(error = %e, "find filesystem node");
            Err("Could not load the purchase order files directory.".into())
        }
    }
}

/// Filesystem path used when no purchase-order files directory has been saved.
pub const DEFAULT_PURCHASE_ORDER_FILES_PATH: &str = "/Purchase Orders";

const DEFAULT_PURCHASE_ORDER_FILES_NAME: &str = "Purchase Orders";

/// File-picker URL that opens inside `directory_id`.
/// Empty when `directory_id` is missing, so the picker starts at the filesystem root.
pub fn purchase_order_file_select_url(directory_id: Option<i64>) -> String {
    match directory_id.filter(|&id| id > 0) {
        Some(id) => VNodeFileSelectInRouteTag::new(id).url(),
        None => String::new(),
    }
}

/// Saved directory when it is a folder; otherwise `/Purchase Orders`.
pub async fn effective_purchase_order_files_directory_id(
    db: &DatabaseConnection,
    configured: Option<i64>,
) -> Option<i64> {
    if let Some(id) = configured.filter(|&id| id > 0)
        && let Ok(Some(node)) = VNodeEntity::find_by_id(id).one(db).await
        && node.is_directory
    {
        return Some(node.id);
    }
    if configured.filter(|&id| id > 0).is_some() {
        return configured;
    }
    default_purchase_order_files_directory_id(db).await
}

async fn default_purchase_order_files_directory_id(db: &DatabaseConnection) -> Option<i64> {
    match lariv_plugin_filesystem::node::get_by_path(db, DEFAULT_PURCHASE_ORDER_FILES_PATH).await {
        Ok((Some(node), _)) if node.is_directory => return Some(node.id),
        Ok((Some(node), _)) => {
            tracing::warn!(
                id = node.id,
                path = DEFAULT_PURCHASE_ORDER_FILES_PATH,
                "default purchase order path is not a directory"
            );
            return None;
        }
        Ok((None, _)) => {}
        Err(lariv_plugin_filesystem::node::NodeError::Validation(msg))
            if msg.starts_with("path not found") => {}
        Err(e) => {
            tracing::error!(error = %e, "lookup default purchase order directory");
            return None;
        }
    }
    let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
    match lariv_plugin_filesystem::node::ensure_directory_path(
        db,
        store.as_ref(),
        None,
        &[DEFAULT_PURCHASE_ORDER_FILES_NAME.to_string()],
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "create default purchase order directory");
            None
        }
    }
}

async fn backfill_default_po_files_directory(
    db: &DatabaseConnection,
    prefs: &mut preferences::Model,
) {
    if prefs
        .purchase_order_files_directory_id
        .filter(|&id| id > 0)
        .is_some()
    {
        return;
    }
    let Some(id) = default_purchase_order_files_directory_id(db).await else {
        return;
    };
    let now = Utc::now();
    let am = preferences::ActiveModel {
        id: Set(prefs.id),
        purchase_order_files_directory_id: Set(Some(id)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    match am.update(db).await {
        Ok(_) => {
            prefs.purchase_order_files_directory_id = Some(id);
            prefs.updated_at = Some(now);
        }
        Err(e) => {
            tracing::error!(error = %e, directory_id = id, "save default purchase order directory");
        }
    }
}

pub async fn purchase_order_file_select_url_from_prefs(db: &DatabaseConnection) -> String {
    let prefs = load_preferences(db).await;
    let id = match prefs.purchase_order_files_directory_id.filter(|&id| id > 0) {
        Some(id) => match VNodeEntity::find_by_id(id).one(db).await {
            Ok(Some(node)) if node.is_directory => Some(node.id),
            _ => default_purchase_order_files_directory_id(db).await,
        },
        None => default_purchase_order_files_directory_id(db).await,
    };
    purchase_order_file_select_url(id)
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

/// Comma-separated site names keyed by draft invoice id (names sorted).
pub async fn site_names_for_invoices(
    db: &DatabaseConnection,
    draft_invoice_ids: &[i64],
) -> std::collections::HashMap<i64, String> {
    use std::collections::HashMap;

    if draft_invoice_ids.is_empty() {
        return HashMap::new();
    }
    let links = SiteInvoiceLinkEntity::find()
        .filter(site_invoice_link::Column::DraftInvoiceId.is_in(draft_invoice_ids.to_vec()))
        .find_also_related(SiteEntity)
        .all(db)
        .await
        .unwrap_or_default();

    let mut names_by_draft: HashMap<i64, Vec<String>> = HashMap::new();
    for (link, site) in links {
        if let Some(site) = site {
            names_by_draft
                .entry(link.draft_invoice_id)
                .or_default()
                .push(site.name);
        }
    }
    names_by_draft
        .into_iter()
        .map(|(id, mut names)| {
            names.sort();
            (id, names.join(", "))
        })
        .collect()
}

fn invoice_label(id: i64, number: &Option<String>) -> String {
    match number.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(n) => n.to_string(),
        None => format!("#{id}"),
    }
}

/// Group key for collapsing draft/posted (and replacement drafts) of one logical invoice.
fn invoice_group_key(d: &draft_invoice::Model, display_number: &str) -> String {
    if let Some(n) = Some(display_number.trim()).filter(|s| !s.is_empty())
        && !n.starts_with('#')
    {
        return n.to_ascii_lowercase();
    }
    if let Some(n) = d.number.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return n.to_ascii_lowercase();
    }
    if let Some(r) = d
        .reference
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return r.to_ascii_lowercase();
    }
    format!("id:{}", d.id)
}

/// One linked invoice on a site detail page, with the columns shown on the invoice hub.
pub struct SiteInvoiceRow {
    pub display_id: i64,
    pub draft_id: i64,
    pub number: String,
    pub href: String,
    pub status: String,
    pub customer: String,
    pub open_balance: String,
    pub open_balance_amount: Option<Decimal>,
    pub date: String,
    pub sort_datetime: Option<DateTime<Utc>>,
    pub delivery_date: String,
    pub delivery_on: Option<NaiveDate>,
    pub untaxed_amount: String,
    pub untaxed: Decimal,
    pub total_amount: String,
    pub total: Decimal,
    pub tax_levied: String,
    pub tax: Decimal,
    pub product_count: String,
    pub product_count_n: u32,
    pub product_ids: Vec<i64>,
    pub final_due_date: String,
    pub final_due: Option<NaiveDate>,
}

struct InvoiceAmountCells {
    untaxed_amount: String,
    untaxed: Decimal,
    total_amount: String,
    total: Decimal,
    tax_levied: String,
    tax: Decimal,
    product_count: String,
    product_count_n: u32,
    final_due_date: String,
    final_due: Option<NaiveDate>,
}

fn invoice_amount_cells(
    metrics: &InvoiceListMetrics,
    fmt: &CurrencyFormat,
    dates: &InvoiceDateFormats,
) -> InvoiceAmountCells {
    InvoiceAmountCells {
        untaxed_amount: fmt.display(metrics.untaxed),
        untaxed: metrics.untaxed,
        total_amount: fmt.display(metrics.total),
        total: metrics.total,
        tax_levied: fmt.display(metrics.tax_levied),
        tax: metrics.tax_levied,
        product_count: metrics.product_count.to_string(),
        product_count_n: metrics.product_count,
        final_due_date: dates.calendar_or_dash(metrics.final_due),
        final_due: metrics.final_due,
    }
}

fn site_invoice_row(
    display_id: i64,
    draft_id: i64,
    number: String,
    href: String,
    status: &str,
    customer: String,
    open_balance: String,
    open_balance_amount: Option<Decimal>,
    date: String,
    sort_datetime: Option<DateTime<Utc>>,
    delivery_date: String,
    delivery_on: Option<NaiveDate>,
    product_ids: Vec<i64>,
    amounts: InvoiceAmountCells,
) -> SiteInvoiceRow {
    SiteInvoiceRow {
        display_id,
        draft_id,
        number,
        href,
        status: status.to_string(),
        customer,
        open_balance,
        open_balance_amount,
        date,
        sort_datetime,
        delivery_date,
        delivery_on,
        untaxed_amount: amounts.untaxed_amount,
        untaxed: amounts.untaxed,
        total_amount: amounts.total_amount,
        total: amounts.total,
        tax_levied: amounts.tax_levied,
        tax: amounts.tax,
        product_count: amounts.product_count,
        product_count_n: amounts.product_count_n,
        product_ids,
        final_due_date: amounts.final_due_date,
        final_due: amounts.final_due,
    }
}

async fn draft_product_ids(db: &DatabaseConnection, draft_id: i64) -> Vec<i64> {
    DraftInvoiceGoodsLineEntity::find()
        .filter(draft_invoice_goods_line::Column::DraftInvoiceId.eq(draft_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|line| line.product_id)
        .collect()
}

async fn posted_product_ids(db: &DatabaseConnection, posted_id: i64) -> Vec<i64> {
    PostedInvoiceGoodsLineEntity::find()
        .filter(posted_invoice_goods_line::Column::PostedInvoiceId.eq(posted_id))
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|line| line.product_id)
        .collect()
}

async fn cancelled_product_ids(db: &DatabaseConnection, cancelled_id: i64) -> Vec<i64> {
    db.query_all_raw(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT product_id FROM cancelled_invoice_goods_lines WHERE cancelled_invoice_id = $1",
        [cancelled_id.into()],
    ))
    .await
    .unwrap_or_default()
    .into_iter()
    .filter_map(|row| row.try_get("", "product_id").ok())
    .collect()
}

async fn party_name(db: &DatabaseConnection, customer_id: i64) -> String {
    PartyLabels::one(db, load_bill_to(db, customer_id).await).await
}

async fn payment_datetime(
    db: &DatabaseConnection,
    payment_id: i64,
    tz: &str,
    dates: &InvoiceDateFormats,
) -> (String, Option<DateTime<Utc>>) {
    let Ok(Some(payment)) = PaymentEntity::find_by_id(payment_id).one(db).await else {
        return ("—".into(), None);
    };
    (dates.datetime(payment.datetime, tz), Some(payment.datetime))
}

/// Furthest lifecycle state for a linked draft, with hub list columns filled in.
async fn resolve_site_invoice(
    db: &DatabaseConnection,
    draft: &draft_invoice::Model,
    tz: &str,
    dates: &InvoiceDateFormats,
    default_currency: &CurrencyFormat,
) -> (SiteInvoiceRow, u8) {
    let posted = PostedInvoiceEntity::find()
        .filter(posted_invoice::Column::DraftInvoiceId.eq(draft.id))
        .one(db)
        .await
        .ok()
        .flatten();
    let Some(posted) = posted else {
        let metrics = draft_invoice_list_metrics(db, draft.id, tz).await;
        let customer = party_name(db, draft.customer_id).await;
        return (
            site_invoice_row(
                draft.id,
                draft.id,
                invoice_label(draft.id, &draft.number),
                DraftInvoiceDetailRouteTag::new(draft.id).url(),
                "Draft",
                customer,
                String::new(),
                None,
                dates.datetime(draft.datetime, tz),
                Some(draft.datetime),
                dates.calendar_or_dash(draft.delivery_date),
                draft.delivery_date,
                draft_product_ids(db, draft.id).await,
                invoice_amount_cells(&metrics, default_currency, dates),
            ),
            2,
        );
    };

    let currency = load_journal_currency_format(db, posted.journal_id).await;
    let number = if posted.number.trim().is_empty() {
        invoice_label(draft.id, &draft.number)
    } else {
        posted.number.clone()
    };

    if let Ok(Some(cancelled)) = CancelledInvoiceEntity::find()
        .filter(cancelled_invoice::Column::PostedInvoiceId.eq(posted.id))
        .one(db)
        .await
    {
        let metrics = cancelled_invoice_list_metrics(db, cancelled.id).await;
        let currency = load_journal_currency_format(db, cancelled.journal_id).await;
        let customer = party_name(db, cancelled.customer_id).await;
        let number = if cancelled.number.trim().is_empty() {
            number
        } else {
            cancelled.number.clone()
        };
        return (
            site_invoice_row(
                cancelled.id,
                draft.id,
                number,
                CancelledInvoiceDetailRouteTag::new(cancelled.id).url(),
                "Cancelled",
                customer,
                String::new(),
                None,
                dates.datetime(cancelled.datetime, tz),
                Some(cancelled.datetime),
                dates.calendar_or_dash(cancelled.delivery_date),
                cancelled.delivery_date,
                cancelled_product_ids(db, cancelled.id).await,
                invoice_amount_cells(&metrics, &currency, dates),
            ),
            1,
        );
    }

    let metrics = posted_invoice_list_metrics(db, posted.id).await;
    let product_ids = posted_product_ids(db, posted.id).await;
    let open = posted_invoice_open_balance(db, posted.id)
        .await
        .unwrap_or(Decimal::ZERO);
    let customer = party_name(db, posted.customer_id).await;
    let open_balance = currency.display(open);

    if let Ok(Some(paid)) = PaidInvoiceEntity::find()
        .filter(paid_invoice::Column::PostedInvoiceId.eq(posted.id))
        .one(db)
        .await
    {
        let (date, sort_datetime) = payment_datetime(db, paid.payment_id, tz, dates).await;
        return (
            site_invoice_row(
                paid.id,
                draft.id,
                number,
                PaidInvoiceDetailRouteTag::new(paid.id).url(),
                "Paid",
                customer,
                open_balance,
                Some(open),
                date,
                sort_datetime,
                dates.calendar_or_dash(posted.delivery_date),
                posted.delivery_date,
                product_ids.clone(),
                invoice_amount_cells(&metrics, &currency, dates),
            ),
            5,
        );
    }
    if let Ok(Some(partial)) = PartiallyPaidInvoiceEntity::find()
        .filter(partially_paid_invoice::Column::PostedInvoiceId.eq(posted.id))
        .one(db)
        .await
    {
        let (date, sort_datetime) = payment_datetime(db, partial.payment_id, tz, dates).await;
        return (
            site_invoice_row(
                partial.id,
                draft.id,
                number,
                PartiallyPaidInvoiceDetailRouteTag::new(partial.id).url(),
                "Partial",
                customer,
                open_balance,
                Some(open),
                date,
                sort_datetime,
                dates.calendar_or_dash(posted.delivery_date),
                posted.delivery_date,
                product_ids.clone(),
                invoice_amount_cells(&metrics, &currency, dates),
            ),
            4,
        );
    }
    (
        site_invoice_row(
            posted.id,
            draft.id,
            number,
            PostedInvoiceDetailRouteTag::new(posted.id).url(),
            "Posted",
            customer,
            open_balance,
            Some(open),
            dates.datetime(posted.datetime, tz),
            Some(posted.datetime),
            dates.calendar_or_dash(posted.delivery_date),
            posted.delivery_date,
            product_ids,
            invoice_amount_cells(&metrics, &currency, dates),
        ),
        3,
    )
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
    tz: &str,
) -> Vec<SiteInvoiceRow> {
    let mut drafts = load_invoices_for_site(db, site_id).await;
    drafts.sort_by_key(|a| std::cmp::Reverse(a.id));
    let dates = load_invoice_date_formats(db).await;
    let default_currency = load_default_currency_format(db).await;

    // One row per logical invoice: prefer the furthest lifecycle state (e.g. Posted over Draft).
    let mut best: HashMap<String, (SiteInvoiceRow, u8)> = HashMap::new();
    for d in drafts {
        let (row, rank) = resolve_site_invoice(db, &d, tz, &dates, &default_currency).await;
        let key = invoice_group_key(&d, &row.number);
        match best.get(&key) {
            Some((existing, existing_rank))
                if *existing_rank > rank
                    || (*existing_rank == rank && existing.draft_id >= d.id) => {}
            _ => {
                best.insert(key, (row, rank));
            }
        }
    }

    let mut out: Vec<_> = best.into_values().map(|(row, _)| row).collect();
    out.sort_by_key(|row| std::cmp::Reverse(row.draft_id));
    out
}

pub async fn load_purchase_orders_for_site(
    db: &DatabaseConnection,
    site_id: i64,
) -> Vec<purchase_order::Model> {
    PurchaseOrderEntity::find()
        .filter(purchase_order::Column::SiteId.eq(site_id))
        .order_by_desc(purchase_order::Column::Id)
        .all(db)
        .await
        .unwrap_or_default()
}

pub async fn purchase_order_items_for_site(
    db: &DatabaseConnection,
    site_id: i64,
) -> Vec<ManyToManyItem> {
    load_purchase_orders_for_site(db, site_id)
        .await
        .into_iter()
        .map(|po| ManyToManyItem::new(po.id.to_string(), po.number))
        .collect()
}

pub async fn purchase_order_items_from_ids(
    db: &DatabaseConnection,
    ids: &[i64],
) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let models = PurchaseOrderEntity::find()
        .filter(purchase_order::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            models
                .iter()
                .find(|po| po.id == *id)
                .map(|po| ManyToManyItem::new(po.id.to_string(), po.number.clone()))
        })
        .collect()
}

pub async fn sync_site_purchase_orders<C: ConnectionTrait>(
    db: &C,
    site_id: i64,
    customer_id: i64,
    po_ids: &[i64],
) -> Result<(), String> {
    let current = PurchaseOrderEntity::find()
        .filter(purchase_order::Column::SiteId.eq(site_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;
    let current_ids: std::collections::BTreeSet<i64> = current.iter().map(|po| po.id).collect();
    let desired: std::collections::BTreeSet<i64> =
        po_ids.iter().copied().filter(|&id| id > 0).collect();
    if current_ids.difference(&desired).next().is_some() {
        return Err(
            "Purchase orders cannot be removed from a site here. Edit the purchase order to assign a different site."
                .into(),
        );
    }
    let now = Utc::now();
    for &po_id in &desired {
        if current_ids.contains(&po_id) {
            continue;
        }
        let Some(existing) = PurchaseOrderEntity::find_by_id(po_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
        else {
            continue;
        };
        purchase_order::ActiveModel {
            id: Set(existing.id),
            site_id: Set(site_id),
            customer_id: Set(customer_id),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .update(db)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
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

/// Link a draft invoice to a site without removing the site's other invoice links.
pub async fn link_site_invoice<C: ConnectionTrait>(
    db: &C,
    site_id: i64,
    draft_invoice_id: i64,
) -> Result<(), String> {
    if site_id <= 0 || draft_invoice_id <= 0 {
        return Err("site_id and draft_invoice_id are required".into());
    }
    let exists = SiteInvoiceLinkEntity::find()
        .filter(site_invoice_link::Column::SiteId.eq(site_id))
        .filter(site_invoice_link::Column::DraftInvoiceId.eq(draft_invoice_id))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;
    if exists.is_some() {
        return Ok(());
    }
    site_invoice_link::ActiveModel {
        site_id: Set(site_id),
        draft_invoice_id: Set(draft_invoice_id),
    }
    .insert(db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Remove a draft invoice link from a site without touching the site's other invoice links.
pub async fn unlink_site_invoice<C: ConnectionTrait>(
    db: &C,
    site_id: i64,
    draft_invoice_id: i64,
) -> Result<(), String> {
    if site_id <= 0 || draft_invoice_id <= 0 {
        return Err("site_id and draft_invoice_id are required".into());
    }
    let res = SiteInvoiceLinkEntity::delete_many()
        .filter(site_invoice_link::Column::SiteId.eq(site_id))
        .filter(site_invoice_link::Column::DraftInvoiceId.eq(draft_invoice_id))
        .exec(db)
        .await
        .map_err(|e| e.to_string())?;
    if res.rows_affected == 0 {
        return Err(format!(
            "site {site_id} is not linked to invoice {draft_invoice_id}"
        ));
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

pub async fn site_name(db: &DatabaseConnection, site_id: i64) -> String {
    if site_id <= 0 {
        return String::new();
    }
    lariv_core::web::opt_or_log(SiteEntity::find_by_id(site_id).one(db).await, "find by id")
        .map(|s| s.name)
        .unwrap_or_else(|| format!("#{site_id}"))
}

pub async fn find_site_for_customer(
    db: &DatabaseConnection,
    customer_id: i64,
) -> Option<site::Model> {
    if customer_id <= 0 {
        return None;
    }
    let Ok(sites) = SiteEntity::find()
        .filter(site::Column::CustomerId.eq(customer_id))
        .order_by_asc(site::Column::Id)
        .all(db)
        .await
    else {
        return None;
    };
    match sites.len() {
        0 => None,
        1 => sites.into_iter().next(),
        _ => {
            let today = Utc::now().date_naive();
            let current: Vec<_> = sites
                .iter()
                .filter(|s| crate::logic::site_is_current(s, today))
                .cloned()
                .collect();
            if current.len() == 1 {
                current.into_iter().next()
            } else {
                None
            }
        }
    }
}

pub async fn customer_name(db: &DatabaseConnection, customer_id: i64) -> String {
    if customer_id <= 0 {
        return String::new();
    }
    lariv_core::web::opt_or_log(
        CompanyEntity::find_by_id(customer_id).one(db).await,
        "find by id",
    )
    .map(|c| c.name)
    .unwrap_or_else(|| format!("#{customer_id}"))
}

pub async fn product_name(db: &DatabaseConnection, product_id: Option<i64>) -> String {
    let Some(id) = product_id.filter(|&id| id > 0) else {
        return String::new();
    };
    lariv_core::web::opt_or_log(ProductEntity::find_by_id(id).one(db).await, "find by id")
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
    let mut prefs = if let Ok(Some(p)) = PreferencesEntity::find_by_id(1i64).one(db).await {
        p
    } else {
        let now = Utc::now();
        let am = preferences::ActiveModel {
            id: Set(1),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            gandola_product_id: Set(None),
            tpi_product_id: Set(None),
            dti_product_id: Set(None),
            payment_term_lines_json: Set(Some(default_payment_term_lines_json())),
            gemini_api_key: Set(String::new()),
            gemini_model: Set("gemini-2.5-flash".to_string()),
            purchase_order_files_directory_id: Set(None),
        };
        am.insert(db).await.unwrap_or(preferences::Model {
            id: 1,
            created_at: Some(now),
            updated_at: Some(now),
            gandola_product_id: None,
            tpi_product_id: None,
            dti_product_id: None,
            payment_term_lines_json: Some(default_payment_term_lines_json()),
            gemini_api_key: String::new(),
            gemini_model: "gemini-2.5-flash".to_string(),
            purchase_order_files_directory_id: None,
        })
    };
    backfill_default_po_files_directory(db, &mut prefs).await;
    prefs
}

pub fn opt_string(s: String) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_PURCHASE_ORDER_FILES_PATH, purchase_order_file_select_url};

    #[test]
    fn default_purchase_order_directory_path() {
        assert_eq!(DEFAULT_PURCHASE_ORDER_FILES_PATH, "/Purchase Orders");
    }

    #[test]
    fn file_select_opens_inside_configured_directory() {
        let url = purchase_order_file_select_url(Some(42));
        assert!(url.contains("/filesystem/file-select/in/42"), "{url}");
    }

    #[test]
    fn file_select_stays_unset_without_directory() {
        assert!(purchase_order_file_select_url(None).is_empty());
        assert!(purchase_order_file_select_url(Some(0)).is_empty());
        assert!(purchase_order_file_select_url(Some(-1)).is_empty());
    }
}
