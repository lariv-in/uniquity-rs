use axum::{
    extract::{Multipart, Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait, QueryOrder,
};

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    html_form::{HtmlForm, HtmlFormBody},
    http::Cap,
    picker::respond_picker_select,
    plugins::{
        filesystem::state::FilesystemState,
        finance_invoices::logic::{default_payment_term_lines_json, parse_payment_term_lines_json},
        users::{middleware::RequireAuth, state::AuthContext},
    },
    template::RenderAppPane,
    web::{
        Htmx, QueryPage, html_built_page_or_app_layout, html_built_page_with_slots,
        respond_create_modal_done_fk, respond_edit_modal_done,
    },
};

use crate::{
    entities::{
        PurchaseOrderPaymentTermEntity,
        purchase_order::{self, Entity as PurchaseOrderEntity},
    },
    forms::{PurchaseOrderForm, PurchaseOrderFromPdfForm},
    handlers::ModalNameQuery,
    keys::{
        PurchaseOrderCreateModalKey, PurchaseOrderEditModalKey, PurchaseOrderFromPdfModalKey,
        PurchaseOrderSelectModalKey, PurchaseOrderSelectTableKey, PurchaseOrderTableKey,
    },
    po_from_pdf::{extract_purchase_order_from_pdf, form_from_extracted, store_purchase_order_pdf},
    po_lines::{
        default_po_lines_json, load_po_line_displays, parse_po_lines_json, po_lines_form_json,
        replace_po_lines,
    },
    po_payment_term::{
        payment_term_lines_form_json_for_po_term, upsert_purchase_order_payment_term_lines,
    },
    routes::PurchaseOrderDetailRouteTag,
    scope::{
        apply_number_filter_purchase_orders, customer_name, find_purchase_order_scoped,
        find_site_scoped, is_superuser, load_preferences, opt_string, parse_optional_i64,
        scope_purchase_orders, site_name, vnode_name,
    },
    state::GandolaManagerState,
    templates::{
        PoLineRow, PurchaseOrderCreateModalPage, PurchaseOrderDetailPage,
        PurchaseOrderEditModalPage, PurchaseOrderFromPdfModalPage, PurchaseOrderListPage,
        PurchaseOrderRow, PurchaseOrderSelectPage,
    },
};

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;
const LIST_URL: &str = "/gandola/purchase-orders/";
const MAX_PO_PDF_BYTES: usize = 15 * 1024 * 1024;

#[derive(Debug, serde::Deserialize, Default)]
pub struct PurchaseOrderListQuery {
    #[serde(default, rename = "Number", alias = "number")]
    pub number: Option<String>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: QueryPage,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct PurchaseOrderSelectQuery {
    #[serde(flatten)]
    pub filter: PurchaseOrderListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

fn format_date(d: chrono::NaiveDate) -> String {
    lariv_rs::datetime::format_date(d)
}

async fn po_to_row(
    db: &sea_orm::DatabaseConnection,
    po: purchase_order::Model,
) -> PurchaseOrderRow {
    PurchaseOrderRow {
        id: po.id,
        number: po.number,
        date: format_date(po.date),
        customer_name: customer_name(db, po.customer_id).await,
        site_name: site_name(db, po.site_id).await,
    }
}

async fn query_purchase_orders(
    db: &sea_orm::DatabaseConnection,
    q: &PurchaseOrderListQuery,
    auth: &AuthContext,
    page_size: u32,
) -> ObjectList<PurchaseOrderRow> {
    let mut query = PurchaseOrderEntity::find();
    query = apply_number_filter_purchase_orders(query, q.number.as_deref());
    query = scope_purchase_orders(query, auth);
    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Number DESC") => {
            query.order_by_desc(purchase_order::Column::Number)
        }
        s if s.eq_ignore_ascii_case("Number ASC") || s.eq_ignore_ascii_case("Number") => {
            query.order_by_asc(purchase_order::Column::Number)
        }
        s if s.eq_ignore_ascii_case("Date DESC") => {
            query.order_by_desc(purchase_order::Column::Date)
        }
        s if s.eq_ignore_ascii_case("Date ASC") || s.eq_ignore_ascii_case("Date") => {
            query.order_by_asc(purchase_order::Column::Date)
        }
        _ => query
            .order_by_desc(purchase_order::Column::Date)
            .order_by_desc(purchase_order::Column::Id),
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
        rows.push(po_to_row(db, model).await);
    }
    ObjectList::from_page(rows, page, page_size, total)
}

async fn number_taken(
    db: &sea_orm::DatabaseConnection,
    number: &str,
    except_id: Option<i64>,
) -> bool {
    crate::po_persist::purchase_order_number_taken(db, number, except_id).await
}

struct PoFormContext {
    customer_display: String,
    site_display: String,
    file_display: String,
}

async fn load_form_context(
    db: &sea_orm::DatabaseConnection,
    customer_id: i64,
    site_id: i64,
    file_id: &str,
) -> PoFormContext {
    PoFormContext {
        customer_display: if customer_id > 0 {
            customer_name(db, customer_id).await
        } else {
            String::new()
        },
        site_display: if site_id > 0 {
            site_name(db, site_id).await
        } else {
            String::new()
        },
        file_display: vnode_name(db, parse_optional_i64(file_id)).await,
    }
}

fn clone_form(form: &PurchaseOrderForm) -> PurchaseOrderForm {
    PurchaseOrderForm {
        number: form.number.clone(),
        date: form.date.clone(),
        customer_id: form.customer_id,
        site_id: form.site_id,
        file_id: form.file_id.clone(),
        payment_term_lines_json: form.payment_term_lines_json.clone(),
        po_lines_json: form.po_lines_json.clone(),
        billing_address: form.billing_address.clone(),
        shipping_address: form.shipping_address.clone(),
    }
}

fn empty_form() -> PurchaseOrderForm {
    PurchaseOrderForm {
        number: String::new(),
        date: lariv_rs::datetime::format_date(Utc::now().date_naive()),
        customer_id: 0,
        site_id: 0,
        file_id: String::new(),
        payment_term_lines_json: default_payment_term_lines_json(),
        po_lines_json: default_po_lines_json(),
        billing_address: String::new(),
        shipping_address: String::new(),
    }
}

fn validate_form(form: &PurchaseOrderForm) -> Result<(chrono::NaiveDate, String), String> {
    crate::po_persist::validate_purchase_order_form(form)
}

async fn resolve_site_and_customer(
    db: &sea_orm::DatabaseConnection,
    site_id: i64,
    customer_id: i64,
) -> Result<(i64, i64), String> {
    crate::po_persist::resolve_site_and_customer(db, site_id, customer_id).await
}

pub async fn list(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PurchaseOrderListQuery>,
) -> maud::Markup {
    let purchase_orders = query_purchase_orders(&state.db, &q, &ctx, PAGE_SIZE).await;
    let page = PurchaseOrderListPage {
        purchase_orders,
        filter_number: q.number.clone().unwrap_or_default(),
        sort: q.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        can_edit: is_superuser(&ctx),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<PurchaseOrderTableKey>() {
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

pub async fn detail(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(po) = find_purchase_order_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(LIST_URL).into_response();
    };
    let lines = load_po_line_displays(&state.db, po.id).await;
    let page = PurchaseOrderDetailPage {
        id: po.id,
        number: po.number,
        date: format_date(po.date),
        customer_id: po.customer_id,
        customer_name: customer_name(&state.db, po.customer_id).await,
        site_id: po.site_id,
        site_name: site_name(&state.db, po.site_id).await,
        file_id: po.file_id,
        file_name: vnode_name(&state.db, po.file_id).await,
        billing_address: po.billing_address.unwrap_or_default(),
        shipping_address: po.shipping_address.unwrap_or_default(),
        lines: lines
            .into_iter()
            .map(|l| PoLineRow {
                item_code: l.item_code,
                description: l.description,
                unit: l.unit,
                delivery_date: l.delivery_date,
                quantity: l.quantity,
                rate: l.rate,
            })
            .collect(),
        can_edit: is_superuser(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

async fn create_modal_from_form(
    db: &sea_orm::DatabaseConnection,
    form: &PurchaseOrderForm,
    form_name: String,
    refresh_table: String,
    target_input: String,
    error: String,
) -> PurchaseOrderCreateModalPage {
    let ctx = load_form_context(db, form.customer_id, form.site_id, &form.file_id).await;
    PurchaseOrderCreateModalPage {
        form_name,
        refresh_table,
        target_input,
        form: clone_form(form),
        customer_display: ctx.customer_display,
        site_display: ctx.site_display,
        file_display: ctx.file_display,
        error,
    }
}

pub async fn create_get(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let page = create_modal_from_form(
        &state.db,
        &empty_form(),
        q.form_name(),
        q.refresh_table(),
        q.target_input(),
        String::new(),
    )
    .await;
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<PurchaseOrderForm>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let form_name = q.form_name();
    let refresh_table = q.refresh_table();
    let target_input = q.target_input();
    match crate::po_persist::persist_new_purchase_order(&state.db, &form, &ctx.timezone).await {
        Ok(saved) => respond_create_modal_done_fk::<PurchaseOrderCreateModalKey>(
            &htmx,
            &refresh_table,
            &PurchaseOrderDetailRouteTag::new(saved.id).url(),
            saved.id,
            &saved.number,
            &target_input,
        ),
        Err(e) => {
            let page =
                create_modal_from_form(&state.db, &form, form_name, refresh_table, target_input, e)
                    .await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

fn from_pdf_modal(
    site_id: i64,
    q: &ModalNameQuery,
    error: String,
) -> PurchaseOrderFromPdfModalPage {
    PurchaseOrderFromPdfModalPage {
        site_id,
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        error,
    }
}

pub async fn from_pdf_get(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    if find_site_scoped(&state.db, id, &ctx).await.is_none() {
        return Redirect::to("/gandola/sites/").into_response();
    }
    html_built_page_with_slots(
        &from_pdf_modal(id, &q, String::new()),
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
    .into_response()
}

pub async fn from_pdf_post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(fs): Cap<FilesystemState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    multipart: Multipart,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let Some(site) = find_site_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/gandola/sites/").into_response();
    };
    let parsed = match PurchaseOrderFromPdfForm::from_multipart(multipart).await {
        Ok(p) => p,
        Err(e) => {
            let page = from_pdf_modal(id, &q, e.to_string());
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let filename = parsed.pdf.filename().to_string();
    let content_type = parsed.pdf.content_type().unwrap_or("").to_string();
    let bytes = match parsed.pdf.into_bytes().await {
        Ok(b) => b,
        Err(e) => {
            let page = from_pdf_modal(id, &q, e.to_string());
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    if bytes.len() > MAX_PO_PDF_BYTES {
        let page = from_pdf_modal(id, &q, "PDF is too large (max 15 MB)".into());
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
            .into_response();
    }
    let looks_like_pdf = bytes.starts_with(b"%PDF")
        || content_type.eq_ignore_ascii_case("application/pdf")
        || filename
            .rsplit('.')
            .next()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"));
    if !looks_like_pdf {
        let page = from_pdf_modal(id, &q, "Upload a PDF file".into());
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
            .into_response();
    }
    let prefs = load_preferences(&state.db).await;
    let extracted =
        match extract_purchase_order_from_pdf(&prefs.gemini_api_key, &prefs.gemini_model, &bytes)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                let page = from_pdf_modal(id, &q, e);
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
        };
    let file_id = match store_purchase_order_pdf(&fs, &filename, bytes.to_vec()).await {
        Ok(id) => id,
        Err(e) => {
            let page = from_pdf_modal(id, &q, format!("Could not store the PDF: {e}"));
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let form = form_from_extracted(&extracted, site.customer_id, site.id, file_id);
    match crate::po_persist::persist_new_purchase_order(&state.db, &form, &ctx.timezone).await {
        Ok(saved) => respond_create_modal_done_fk::<PurchaseOrderFromPdfModalKey>(
            &htmx,
            &q.refresh_table(),
            &PurchaseOrderDetailRouteTag::new(saved.id).url(),
            saved.id,
            &saved.number,
            &q.target_input(),
        ),
        Err(e) => {
            let page = create_modal_from_form(
                &state.db,
                &form,
                "gandola_manager.PurchaseOrderCreateForm".into(),
                q.refresh_table(),
                q.target_input(),
                e,
            )
            .await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

async fn form_from_model(
    db: &sea_orm::DatabaseConnection,
    po: &purchase_order::Model,
    tz: &str,
) -> PurchaseOrderForm {
    PurchaseOrderForm {
        number: po.number.clone(),
        date: format_date(po.date),
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

async fn edit_modal_from_form(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    form: &PurchaseOrderForm,
    form_name: String,
    error: String,
) -> PurchaseOrderEditModalPage {
    let ctx = load_form_context(db, form.customer_id, form.site_id, &form.file_id).await;
    PurchaseOrderEditModalPage {
        id,
        form_name,
        form: clone_form(form),
        customer_display: ctx.customer_display,
        site_display: ctx.site_display,
        file_display: ctx.file_display,
        error,
    }
}

pub async fn edit_get(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let Some(po) = find_purchase_order_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(LIST_URL).into_response();
    };
    let form = form_from_model(&state.db, &po, &ctx.timezone).await;
    let page = edit_modal_from_form(&state.db, id, &form, q.form_name(), String::new()).await;
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<PurchaseOrderForm>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let Some(existing) = find_purchase_order_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(LIST_URL).into_response();
    };
    let form_name = q.form_name();
    let (date, number) = match validate_form(&form) {
        Ok(v) => v,
        Err(e) => {
            let page = edit_modal_from_form(&state.db, id, &form, form_name, e).await;
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let (site_id, customer_id) =
        match resolve_site_and_customer(&state.db, form.site_id, form.customer_id).await {
            Ok(v) => v,
            Err(e) => {
                let page = edit_modal_from_form(&state.db, id, &form, form_name, e).await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
        };
    if number_taken(&state.db, &number, Some(id)).await {
        let page = edit_modal_from_form(
            &state.db,
            id,
            &form,
            form_name,
            "number must be unique".into(),
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
            .into_response();
    }
    let lines = match parse_po_lines_json(&form.po_lines_json) {
        Ok(l) => l,
        Err(e) => {
            let page = edit_modal_from_form(&state.db, id, &form, form_name, e).await;
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let term_lines = match parse_payment_term_lines_json(&form.payment_term_lines_json) {
        Ok(l) => l,
        Err(e) => {
            let page = edit_modal_from_form(&state.db, id, &form, form_name, e).await;
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let term = match upsert_purchase_order_payment_term_lines(
        &state.db,
        existing.payment_term_id,
        &term_lines,
        &ctx.timezone,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            let page = edit_modal_from_form(&state.db, id, &form, form_name, e).await;
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
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
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = replace_po_lines(&state.db, id, &lines).await {
                let page = edit_modal_from_form(&state.db, id, &form, form_name, e).await;
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                    .into_response();
            }
            respond_edit_modal_done::<PurchaseOrderEditModalKey>(
                &htmx,
                &PurchaseOrderDetailRouteTag::new(id).url(),
            )
        }
        Err(e) => {
            let page = edit_modal_from_form(&state.db, id, &form, form_name, e.to_string()).await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn delete_post(
    Cap(state): Cap<GandolaManagerState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    if let Some(po) = find_purchase_order_scoped(&state.db, id, &ctx).await {
        let term_id = po.payment_term_id;
        let _ = PurchaseOrderEntity::delete_by_id(id).exec(&state.db).await;
        if let Some(term_id) = term_id {
            let _ = PurchaseOrderPaymentTermEntity::delete_by_id(term_id)
                .exec(&state.db)
                .await;
        }
    }
    Redirect::to(LIST_URL).into_response()
}

pub async fn select(
    Cap(state): Cap<GandolaManagerState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PurchaseOrderSelectQuery>,
) -> maud::Markup {
    let purchase_orders = query_purchase_orders(&state.db, &q.filter, &ctx, PAGE_SIZE).await;
    let page = PurchaseOrderSelectPage {
        purchase_orders,
        filter_number: q.filter.number.clone().unwrap_or_default(),
        sort: q.filter.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q
            .target_input
            .clone()
            .unwrap_or_else(|| "PurchaseOrders".into()),
        can_edit: is_superuser(&ctx),
    };
    respond_picker_select::<PurchaseOrderSelectTableKey, PurchaseOrderSelectModalKey, _>(
        &htmx, &page,
    )
}
