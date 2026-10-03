use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    sea_query::{Expr, NullOrdering, Order, Query as SeaQuery},
};

use lariv_rs::{
    components::{ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
    plugins::users::{middleware::RequireAuth, state::AuthContext},
    template::RenderAppPane,
    web::{
        Htmx, QueryPage, QueryPageSize, html_built_page_or_app_layout, html_built_page_with_slots,
        respond_create_modal_done_fk, respond_edit_modal_done,
    },
};

use crate::{
    entities::{
        gandola_site_link::{self, Entity as GandolaSiteLinkEntity},
        site::{self, Entity as SiteEntity},
    },
    forms::SiteForm,
    handlers::{
        ModalNameQuery,
        gandolas::{GandolaListQuery, query_gandolas},
        purchase_orders::{PurchaseOrderListQuery, query_purchase_orders},
    },
    keys::{
        SiteCreateModalKey, SiteDeleteModalKey, SiteEditModalKey, SiteFkSelectModalKey,
        SiteFkSelectTableKey, SiteGandolasTableKey, SiteInvoicesTableKey,
        SitePurchaseOrdersTableKey, SiteSelectModalKey, SiteSelectTableKey, SiteTableKey,
    },
    routes::{SiteDefaultRouteTag, SiteDetailRouteTag},
    scope::{
        apply_name_filter_sites, apply_site_id_filter_sites, can_manage, customer_name,
        find_site_scoped, gandola_items_for_site, gandola_items_from_ids, invoice_items_for_site,
        invoice_items_from_ids, load_gandolas_for_site, opt_string, purchase_order_items_for_site,
        purchase_order_items_from_ids, related_invoices_for_site, scope_sites, sync_site_gandolas,
        sync_site_invoices, sync_site_purchase_orders,
    },
    site_status::SiteStatus,
    state::GandolaManagerState,
    templates::{
        ConfirmDeletePage, RelatedInvoice, SiteCreateModalPage, SiteDetailPage, SiteEditModalPage,
        SiteFkSelectPage, SiteListPage, SiteRow, SiteSelectPage,
    },
};

fn list_url() -> String {
    SiteDefaultRouteTag.url()
}

/// Sort sites by the first linked gandola name (sites without gandolas sort as empty).
const GANDOLA_NAME_SORT_EXPR: &str = "COALESCE((SELECT MIN(g.name) FROM gandola_sites gs INNER JOIN gandolas g ON g.id = gs.gandola_id WHERE gs.site_id = sites.id), '')";

#[derive(Debug, serde::Deserialize, Default)]
pub struct SiteListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "SiteId", alias = "site_id")]
    pub site_id: Option<String>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: QueryPage,
    #[serde(default)]
    pub page_size: QueryPageSize,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct SiteDetailQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub gandola_name: Option<String>,
    #[serde(default, rename = "g_sort")]
    pub gandola_sort: Option<String>,
    #[serde(default, rename = "g_page")]
    pub gandola_page: QueryPage,

    #[serde(default, rename = "Number", alias = "number")]
    pub po_number: Option<String>,
    #[serde(default, rename = "po_sort")]
    pub po_sort: Option<String>,
    #[serde(default, rename = "po_page")]
    pub po_page: QueryPage,

    #[serde(default, rename = "InvoiceNumber")]
    pub invoice_number: Option<String>,
    #[serde(default, rename = "InvoiceStatus")]
    pub invoice_status: Option<String>,
    #[serde(default, rename = "inv_sort")]
    pub invoice_sort: Option<String>,
    #[serde(default, rename = "inv_page")]
    pub invoice_page: QueryPage,

    #[serde(default)]
    pub page_size: QueryPageSize,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct SiteSelectQuery {
    #[serde(flatten)]
    pub filter: SiteListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

fn parse_date(s: &str) -> Result<Option<NaiveDate>, &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(None);
    }
    lariv_rs::datetime::parse_date(s)
        .map(Some)
        .ok_or("invalid date")
}

fn format_date(d: Option<NaiveDate>) -> String {
    d.map(lariv_rs::datetime::format_date).unwrap_or_default()
}

async fn site_to_row(db: &sea_orm::DatabaseConnection, s: site::Model) -> SiteRow {
    let gandolas = load_gandolas_for_site(db, s.id).await;
    SiteRow {
        id: s.id,
        name: s.name,
        site_id: s.site_id.unwrap_or_default(),
        address: s.address.unwrap_or_default(),
        remarks: s.remarks.unwrap_or_default(),
        start_date: format_date(s.start_date),
        end_date: format_date(s.end_date),
        status: s.status.as_str().to_string(),
        status_label: s.status.label().to_string(),
        gandola_names: gandolas.into_iter().map(|g| g.name).collect(),
    }
}

pub(crate) async fn query_sites(
    db: &sea_orm::DatabaseConnection,
    q: &SiteListQuery,
    auth: &AuthContext,
    page_size: u32,
    gandola_id: Option<i64>,
    default_start_date_desc: bool,
) -> ObjectList<SiteRow> {
    let mut query = SiteEntity::find();
    query = apply_name_filter_sites(query, q.name.as_deref());
    query = apply_site_id_filter_sites(query, q.site_id.as_deref());
    query = scope_sites(query, auth);
    if let Some(gandola_id) = gandola_id {
        let mut linked = SeaQuery::select();
        linked
            .column(gandola_site_link::Column::SiteId)
            .from(GandolaSiteLinkEntity)
            .and_where(gandola_site_link::Column::GandolaId.eq(gandola_id));
        query = query.filter(site::Column::Id.in_subquery(linked));
    }
    let mut sort = q.sort.as_deref().unwrap_or("").trim().to_string();
    if sort.is_empty() && default_start_date_desc {
        sort = "StartDate DESC".to_string();
    }
    let sort = sort.as_str();
    query = match sort {
        s if s.eq_ignore_ascii_case("Name DESC") => query.order_by_desc(site::Column::Name),
        s if s.eq_ignore_ascii_case("Name ASC") || s.eq_ignore_ascii_case("Name") => {
            query.order_by_asc(site::Column::Name)
        }
        s if s.eq_ignore_ascii_case("SiteId DESC") => query.order_by_desc(site::Column::SiteId),
        s if s.eq_ignore_ascii_case("SiteId ASC") || s.eq_ignore_ascii_case("SiteId") => {
            query.order_by_asc(site::Column::SiteId)
        }
        s if s.eq_ignore_ascii_case("Address DESC") => query.order_by_desc(site::Column::Address),
        s if s.eq_ignore_ascii_case("Address ASC") || s.eq_ignore_ascii_case("Address") => {
            query.order_by_asc(site::Column::Address)
        }
        s if s.eq_ignore_ascii_case("Remarks DESC") => query.order_by_desc(site::Column::Remarks),
        s if s.eq_ignore_ascii_case("Remarks ASC") || s.eq_ignore_ascii_case("Remarks") => {
            query.order_by_asc(site::Column::Remarks)
        }
        s if s.eq_ignore_ascii_case("StartDate DESC") => query
            .order_by_with_nulls(site::Column::StartDate, Order::Desc, NullOrdering::Last)
            .order_by_desc(site::Column::Id),
        s if s.eq_ignore_ascii_case("StartDate ASC") || s.eq_ignore_ascii_case("StartDate") => {
            query.order_by_asc(site::Column::StartDate)
        }
        s if s.eq_ignore_ascii_case("EndDate DESC") => query.order_by_desc(site::Column::EndDate),
        s if s.eq_ignore_ascii_case("EndDate ASC") || s.eq_ignore_ascii_case("EndDate") => {
            query.order_by_asc(site::Column::EndDate)
        }
        s if s.eq_ignore_ascii_case("Status DESC") => query.order_by_desc(site::Column::Status),
        s if s.eq_ignore_ascii_case("Status ASC") || s.eq_ignore_ascii_case("Status") => {
            query.order_by_asc(site::Column::Status)
        }
        s if s.eq_ignore_ascii_case("Gandolas DESC") => {
            query.order_by(Expr::cust(GANDOLA_NAME_SORT_EXPR), Order::Desc)
        }
        s if s.eq_ignore_ascii_case("Gandolas ASC") || s.eq_ignore_ascii_case("Gandolas") => {
            query.order_by(Expr::cust(GANDOLA_NAME_SORT_EXPR), Order::Asc)
        }
        _ => query.order_by_desc(site::Column::Id),
    };
    let page = q.page.get();
    let paginator = query.paginate(db, page_size as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for model in models {
        rows.push(site_to_row(db, model).await);
    }
    ObjectList::from_page(rows, page, page_size, total)
}

pub async fn list(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SiteListQuery>,
) -> maud::Markup {
    let sites = query_sites(&state.db, &q, &ctx, q.page_size.get(), None, false).await;
    let page = SiteListPage {
        sites,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_site_id: q.site_id.clone().unwrap_or_default(),
        sort: q.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        can_edit: can_manage(&ctx),
        page_size: q.page_size.get(),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<SiteTableKey>() {
        return page.render_table();
    }
    if htmx.wants_main_content() {
        return page.render_main().into();
    }
    if htmx.wants_app_layout() {
        return page.render_pane().into();
    }
    html_built_page_with_slots(&page, &chrome, &slot_ctx)
}

fn query_site_invoices(
    mut rows: Vec<(i64, String, String, String, String, DateTime<Utc>)>,
    number: Option<&str>,
    status: Option<&str>,
    sort: &str,
    page: u32,
    page_size: u32,
) -> ObjectList<RelatedInvoice> {
    let number = number.unwrap_or("").trim().to_lowercase();
    let status = status.unwrap_or("").trim();
    rows.retain(|row| {
        (number.is_empty() || row.1.to_lowercase().contains(&number))
            && (status.is_empty() || row.4.eq_ignore_ascii_case(status))
    });
    let sort = if sort.trim().is_empty() {
        "Date DESC"
    } else {
        sort.trim()
    };
    rows.sort_by(|a, b| {
        let ord = if sort.eq_ignore_ascii_case("Number DESC") {
            b.1.cmp(&a.1)
        } else if sort.eq_ignore_ascii_case("Number ASC") || sort.eq_ignore_ascii_case("Number") {
            a.1.cmp(&b.1)
        } else if sort.eq_ignore_ascii_case("Status DESC") {
            b.4.cmp(&a.4)
        } else if sort.eq_ignore_ascii_case("Status ASC") || sort.eq_ignore_ascii_case("Status") {
            a.4.cmp(&b.4)
        } else if sort.eq_ignore_ascii_case("Date ASC") || sort.eq_ignore_ascii_case("Date") {
            a.5.cmp(&b.5)
        } else {
            b.5.cmp(&a.5)
        };
        ord.then(b.0.cmp(&a.0))
    });
    let total = rows.len() as u64;
    let start = (page.saturating_sub(1) as usize).saturating_mul(page_size as usize);
    let items = rows
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .map(|(id, name, href, date, status, _)| RelatedInvoice {
            id,
            name,
            href,
            date,
            status,
        })
        .collect();
    ObjectList::from_page(items, page, page_size, total)
}

pub async fn detail(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Path(id): Path<i64>,
    Query(q): Query<SiteDetailQuery>,
) -> Response {
    let Some(s) = find_site_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&list_url()).into_response();
    };
    let mut gandola_query = GandolaListQuery {
        name: q.gandola_name.clone(),
        sort: q.gandola_sort.clone(),
        ..Default::default()
    };
    gandola_query.page = q.gandola_page;
    gandola_query.page_size = q.page_size;
    let gandolas = query_gandolas(
        &state.db,
        &gandola_query,
        &ctx,
        q.page_size.get(),
        Some(s.id),
    )
    .await;
    let mut po_query = PurchaseOrderListQuery {
        number: q.po_number.clone(),
        sort: q.po_sort.clone(),
        ..Default::default()
    };
    po_query.page = q.po_page;
    po_query.page_size = q.page_size;
    let purchase_orders =
        query_purchase_orders(&state.db, &po_query, &ctx, q.page_size.get(), Some(s.id)).await;
    let invoices = query_site_invoices(
        related_invoices_for_site(&state.db, s.id, &ctx.timezone).await,
        q.invoice_number.as_deref(),
        q.invoice_status.as_deref(),
        q.invoice_sort.as_deref().unwrap_or(""),
        q.invoice_page.get(),
        q.page_size.get(),
    );
    let page = SiteDetailPage {
        id: s.id,
        name: s.name,
        site_id: s.site_id.unwrap_or_default(),
        customer_id: s.customer_id,
        customer_name: customer_name(&state.db, s.customer_id).await,
        status_label: s.status.label().to_string(),
        status: s.status.as_str().to_string(),
        start_date: format_date(s.start_date),
        end_date: format_date(s.end_date),
        address: s.address.unwrap_or_default(),
        remarks: s.remarks.unwrap_or_default(),
        gandolas,
        gandola_filter_name: q.gandola_name.clone().unwrap_or_default(),
        gandola_sort: q.gandola_sort.clone().unwrap_or_default(),
        purchase_orders,
        po_filter_number: q.po_number.clone().unwrap_or_default(),
        po_sort: q.po_sort.clone().unwrap_or_default(),
        invoices,
        invoice_filter_number: q.invoice_number.clone().unwrap_or_default(),
        invoice_filter_status: q.invoice_status.clone().unwrap_or_default(),
        invoice_sort: q.invoice_sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        page_size: q.page_size.get(),
        can_edit: can_manage(&ctx),
    };
    if htmx.targets::<SiteGandolasTableKey>() {
        return page.render_gandolas_table().into_response();
    }
    if htmx.targets::<SitePurchaseOrdersTableKey>() {
        return page.render_purchase_orders_table().into_response();
    }
    if htmx.targets::<SiteInvoicesTableKey>() {
        return page.render_invoices_table().into_response();
    }
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

struct ParsedSite {
    name: String,
    site_id: Option<String>,
    customer_id: i64,
    status: SiteStatus,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    address: Option<String>,
    remarks: Option<String>,
}

fn parse_site_form(form: &SiteForm) -> Result<ParsedSite, String> {
    if form.name.trim().is_empty() {
        return Err("Name is required".into());
    }
    if form.customer_id <= 0 {
        return Err("Customer is required".into());
    }
    let status = SiteStatus::parse(&form.status).unwrap_or_default();
    let start_date = parse_date(&form.start_date).map_err(|e| e.to_string())?;
    let end_date = parse_date(&form.end_date).map_err(|e| e.to_string())?;
    Ok(ParsedSite {
        name: form.name.trim().to_string(),
        site_id: opt_string(form.site_id.clone()),
        customer_id: form.customer_id,
        status,
        start_date,
        end_date,
        address: opt_string(form.address.clone()),
        remarks: opt_string(form.remarks.clone()),
    })
}

async fn create_page_from_form(
    db: &sea_orm::DatabaseConnection,
    form: &SiteForm,
    gandolas: Vec<ManyToManyItem>,
    invoices: Vec<ManyToManyItem>,
    purchase_orders: Vec<ManyToManyItem>,
    form_name: String,
    refresh_table: String,
    target_input: String,
    error: String,
) -> SiteCreateModalPage {
    SiteCreateModalPage {
        form_name,
        refresh_table,
        target_input,
        name: form.name.clone(),
        site_id: form.site_id.clone(),
        customer_id: form.customer_id,
        customer_display: customer_name(db, form.customer_id).await,
        status: form.status.clone(),
        start_date: form.start_date.clone(),
        end_date: form.end_date.clone(),
        address: form.address.clone(),
        remarks: form.remarks.clone(),
        gandolas,
        invoices,
        purchase_orders,
        error,
    }
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !can_manage(&ctx) {
        return Redirect::to(&list_url()).into_response();
    }
    let page = SiteCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        name: String::new(),
        site_id: String::new(),
        customer_id: 0,
        customer_display: String::new(),
        status: SiteStatus::default().as_str().to_string(),
        start_date: String::new(),
        end_date: String::new(),
        address: String::new(),
        remarks: String::new(),
        gandolas: Vec::new(),
        invoices: Vec::new(),
        purchase_orders: Vec::new(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<SiteForm>,
) -> Response {
    if !can_manage(&ctx) {
        return Redirect::to(&list_url()).into_response();
    }
    let gandolas = gandola_items_from_ids(&state.db, &form.gandolas).await;
    let invoices = invoice_items_from_ids(&state.db, &form.invoices).await;
    let purchase_orders = purchase_order_items_from_ids(&state.db, &form.purchase_orders).await;
    let parsed = match parse_site_form(&form) {
        Ok(p) => p,
        Err(e) => {
            let page = create_page_from_form(
                &state.db,
                &form,
                gandolas,
                invoices,
                purchase_orders,
                q.form_name(),
                q.refresh_table(),
                q.target_input(),
                e,
            )
            .await;
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let now = Utc::now();
    let model = site::ActiveModel {
        name: Set(parsed.name),
        site_id: Set(parsed.site_id),
        customer_id: Set(parsed.customer_id),
        status: Set(parsed.status),
        start_date: Set(parsed.start_date),
        end_date: Set(parsed.end_date),
        address: Set(parsed.address),
        remarks: Set(parsed.remarks),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            if let Err(e) = sync_site_gandolas(&state.db, saved.id, &form.gandolas).await {
                let page = create_page_from_form(
                    &state.db,
                    &form,
                    gandolas,
                    invoices,
                    purchase_orders,
                    q.form_name(),
                    q.refresh_table(),
                    q.target_input(),
                    e,
                )
                .await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            if let Err(e) = sync_site_invoices(&state.db, saved.id, &form.invoices).await {
                let page = create_page_from_form(
                    &state.db,
                    &form,
                    gandolas,
                    invoices,
                    purchase_orders,
                    q.form_name(),
                    q.refresh_table(),
                    q.target_input(),
                    e,
                )
                .await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            if let Err(e) = sync_site_purchase_orders(
                &state.db,
                saved.id,
                saved.customer_id,
                &form.purchase_orders,
            )
            .await
            {
                let page = create_page_from_form(
                    &state.db,
                    &form,
                    gandolas,
                    invoices,
                    purchase_orders,
                    q.form_name(),
                    q.refresh_table(),
                    q.target_input(),
                    e,
                )
                .await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            respond_create_modal_done_fk::<SiteCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &SiteDetailRouteTag::new(saved.id).url(),
                saved.id,
                &saved.name,
                &q.target_input(),
            )
        }
        Err(e) => {
            let page = create_page_from_form(
                &state.db,
                &form,
                gandolas,
                invoices,
                purchase_orders,
                q.form_name(),
                q.refresh_table(),
                q.target_input(),
                e.to_string(),
            )
            .await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !can_manage(&ctx) {
        return Redirect::to(&list_url()).into_response();
    }
    let Some(s) = find_site_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&list_url()).into_response();
    };
    let page = SiteEditModalPage {
        id: s.id,
        form_name: q.form_name(),
        name: s.name,
        site_id: s.site_id.unwrap_or_default(),
        customer_id: s.customer_id,
        customer_display: customer_name(&state.db, s.customer_id).await,
        status: s.status.as_str().to_string(),
        start_date: format_date(s.start_date),
        end_date: format_date(s.end_date),
        address: s.address.unwrap_or_default(),
        remarks: s.remarks.unwrap_or_default(),
        gandolas: gandola_items_for_site(&state.db, s.id).await,
        invoices: invoice_items_for_site(&state.db, s.id).await,
        purchase_orders: purchase_order_items_for_site(&state.db, s.id).await,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

async fn edit_page_from_form(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    form: &SiteForm,
    gandolas: Vec<ManyToManyItem>,
    invoices: Vec<ManyToManyItem>,
    purchase_orders: Vec<ManyToManyItem>,
    form_name: String,
    error: String,
) -> SiteEditModalPage {
    SiteEditModalPage {
        id,
        form_name,
        name: form.name.clone(),
        site_id: form.site_id.clone(),
        customer_id: form.customer_id,
        customer_display: customer_name(db, form.customer_id).await,
        status: form.status.clone(),
        start_date: form.start_date.clone(),
        end_date: form.end_date.clone(),
        address: form.address.clone(),
        remarks: form.remarks.clone(),
        gandolas,
        invoices,
        purchase_orders,
        error,
    }
}

pub async fn edit_post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<SiteForm>,
) -> Response {
    if !can_manage(&ctx) {
        return Redirect::to(&list_url()).into_response();
    }
    let Some(existing) = find_site_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&list_url()).into_response();
    };
    let gandolas = gandola_items_from_ids(&state.db, &form.gandolas).await;
    let invoices = invoice_items_from_ids(&state.db, &form.invoices).await;
    let purchase_orders = purchase_order_items_from_ids(&state.db, &form.purchase_orders).await;
    let parsed = match parse_site_form(&form) {
        Ok(p) => p,
        Err(e) => {
            let page = edit_page_from_form(
                &state.db,
                id,
                &form,
                gandolas,
                invoices,
                purchase_orders,
                q.form_name(),
                e,
            )
            .await;
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let now = Utc::now();
    let model = site::ActiveModel {
        id: Set(existing.id),
        name: Set(parsed.name),
        site_id: Set(parsed.site_id),
        customer_id: Set(parsed.customer_id),
        status: Set(parsed.status),
        start_date: Set(parsed.start_date),
        end_date: Set(parsed.end_date),
        address: Set(parsed.address),
        remarks: Set(parsed.remarks),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = sync_site_gandolas(&state.db, id, &form.gandolas).await {
                let page = edit_page_from_form(
                    &state.db,
                    id,
                    &form,
                    gandolas,
                    invoices,
                    purchase_orders,
                    q.form_name(),
                    e,
                )
                .await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            if let Err(e) = sync_site_invoices(&state.db, id, &form.invoices).await {
                let page = edit_page_from_form(
                    &state.db,
                    id,
                    &form,
                    gandolas,
                    invoices,
                    purchase_orders,
                    q.form_name(),
                    e,
                )
                .await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            if let Err(e) =
                sync_site_purchase_orders(&state.db, id, parsed.customer_id, &form.purchase_orders)
                    .await
            {
                let page = edit_page_from_form(
                    &state.db,
                    id,
                    &form,
                    gandolas,
                    invoices,
                    purchase_orders,
                    q.form_name(),
                    e,
                )
                .await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            respond_edit_modal_done::<SiteEditModalKey>(&htmx, &SiteDetailRouteTag::new(id).url())
        }
        Err(e) => {
            let page = edit_page_from_form(
                &state.db,
                id,
                &form,
                gandolas,
                invoices,
                purchase_orders,
                q.form_name(),
                e.to_string(),
            )
            .await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
    Path(id): Path<i64>,
) -> maud::Markup {
    let page = ConfirmDeletePage {
        modal_uid: SiteDeleteModalKey::ID.to_string(),
        message: "Are you sure you want to delete this site?".into(),
        form_name: q
            .name
            .clone()
            .unwrap_or_else(|| "gandola_manager.SiteDeleteForm".into()),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn delete_post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !can_manage(&ctx) {
        return Redirect::to(&list_url()).into_response();
    }
    if find_site_scoped(&state.db, id, &ctx).await.is_none() {
        return Redirect::to(&list_url()).into_response();
    }
    match crate::site_persist::delete_site(&state.db, id).await {
        Ok(()) => htmx.redirect(&list_url()),
        Err(e) => {
            tracing::error!(error = %e, id, "failed to delete site");
            let page = ConfirmDeletePage {
                modal_uid: SiteDeleteModalKey::ID.to_string(),
                message: "Are you sure you want to delete this site?".into(),
                form_name: "gandola_manager.SiteDeleteForm".into(),
                id,
                error: e,
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<GandolaManagerState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SiteSelectQuery>,
) -> maud::Markup {
    let sites = query_sites(
        &state.db,
        &q.filter,
        &ctx,
        q.filter.page_size.get(),
        None,
        true,
    )
    .await;
    let page = SiteSelectPage {
        sites,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_site_id: q.filter.site_id.clone().unwrap_or_default(),
        sort: q.filter.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.clone().unwrap_or_else(|| "Sites".into()),
        can_edit: can_manage(&ctx),
        page_size: q.filter.page_size.get(),
    };
    respond_picker_select::<SiteSelectTableKey, SiteSelectModalKey, _>(&htmx, &page)
}

pub async fn fk_select(
    Cap(state): Cap<GandolaManagerState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SiteSelectQuery>,
) -> maud::Markup {
    let sites = query_sites(
        &state.db,
        &q.filter,
        &ctx,
        q.filter.page_size.get(),
        None,
        false,
    )
    .await;
    let page = SiteFkSelectPage {
        sites,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_site_id: q.filter.site_id.clone().unwrap_or_default(),
        sort: q.filter.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.clone().unwrap_or_else(|| "SiteID".into()),
        can_edit: can_manage(&ctx),
        page_size: q.filter.page_size.get(),
    };
    respond_picker_select::<SiteFkSelectTableKey, SiteFkSelectModalKey, _>(&htmx, &page)
}
