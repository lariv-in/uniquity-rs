use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait, QueryOrder};
use serde::Deserialize;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    picker::respond_picker_select,
    plugins::users::{middleware::RequireAuth, state::AuthContext},
    template::RenderAppPane,
    web::{Htmx, QueryPage, html_built_page_or_app_layout, html_built_page_with_slots},
};

use uniquity_common::require_superuser;

use crate::{
    entities::customer::{self, Entity as CustomerEntity},
    forms::CustomerForm,
    keys::{CustomerSelectModalKey, CustomerSelectTableKey, CustomerTableKey},
    routes::{CustomerDetailRouteTag, CustomerEditGetRouteTag},
    scope::{apply_customer_filters, find_customer_scoped, scope_customers},
    state::CustomerState,
    templates::{
        CustomerDetailPage, CustomerFormPage, CustomerListPage, CustomerRow,
        CustomerSelectPage,
    },
};

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;

#[derive(Debug, Deserialize, Default)]
pub struct CustomerListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "Email", alias = "email")]
    pub email: Option<String>,
    #[serde(default)]
    pub page: QueryPage,
}

#[derive(Debug, Deserialize, Default)]
pub struct CustomerSelectQuery {
    #[serde(flatten)]
    pub filter: CustomerListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

fn opt_string(s: String) -> Option<String> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

fn customer_address_fields(customer: &customer::Model) -> (String, String, String, String, String) {
    (
        customer.address_line_1.clone().unwrap_or_default(),
        customer.address_line_2.clone().unwrap_or_default(),
        customer.city.clone().unwrap_or_default(),
        customer.pincode.clone().unwrap_or_default(),
        customer.state.clone().unwrap_or_default(),
    )
}

async fn query_customers(
    db: &sea_orm::DatabaseConnection,
    q: &CustomerListQuery,
    auth: &AuthContext,
    page_size: u32,
) -> (Vec<customer::Model>, u32, u64) {
    let mut query = CustomerEntity::find();
    query = apply_customer_filters(query, q.name.as_deref(), q.email.as_deref());
    query = scope_customers(query, auth);
    query = query
        .order_by_desc(customer::Column::CreatedAt)
        .order_by_desc(customer::Column::Id);

    let page = q.page.get();
    let paginator = query.paginate(db, page_size as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    (models, page, total)
}

fn model_to_row(c: customer::Model) -> CustomerRow {
    CustomerRow {
        id: c.id,
        name: c.name,
        email: c.email.unwrap_or_default(),
        phone: c.phone.unwrap_or_default(),
        gstin: c.gstin.unwrap_or_default(),
    }
}

async fn load_customer_rows(
    db: &sea_orm::DatabaseConnection,
    q: &CustomerListQuery,
    auth: &AuthContext,
    page_size: u32,
) -> ObjectList<CustomerRow> {
    let (models, page, total) = query_customers(db, q, auth, page_size).await;
    let rows: Vec<CustomerRow> = models.into_iter().map(model_to_row).collect();
    ObjectList::from_page(rows, page, page_size, total)
}

pub async fn list(
    Cap(state): Cap<CustomerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<CustomerListQuery>,
) -> maud::Markup {
    let customers = load_customer_rows(&state.db, &q, &ctx, PAGE_SIZE).await;
    let page = CustomerListPage {
        customers,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_email: q.email.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        can_edit: require_superuser(&ctx),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<CustomerTableKey>() {
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
    Cap(state): Cap<CustomerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(customer) = find_customer_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/finance-customers/").into_response();
    };
    let (address_line_1, address_line_2, city, pincode, state) = customer_address_fields(&customer);
    let page = CustomerDetailPage {
        id: customer.id,
        name: customer.name,
        address_line_1,
        address_line_2,
        city,
        pincode,
        state,
        gstin: customer.gstin.unwrap_or_default(),
        pan: customer.pan.unwrap_or_default(),
        phone: customer.phone.unwrap_or_default(),
        email: customer.email.unwrap_or_default(),
        website: customer.website.unwrap_or_default(),
        can_edit: require_superuser(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-customers/").into_response();
    }
    let page = CustomerFormPage {
        id: 0,
        name: String::new(),
        address_line_1: String::new(),
        address_line_2: String::new(),
        city: String::new(),
        pincode: String::new(),
        state: String::new(),
        gstin: String::new(),
        pan: String::new(),
        phone: String::new(),
        email: String::new(),
        website: String::new(),
        is_edit: false,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<CustomerState>,
    RequireAuth(ctx): RequireAuth,
    Form(form): Form<CustomerForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-customers/").into_response();
    }
    let now = Utc::now();
    let model = customer::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        name: Set(form.name),
        address_line_1: Set(opt_string(form.address_line_1)),
        address_line_2: Set(opt_string(form.address_line_2)),
        city: Set(opt_string(form.city)),
        pincode: Set(opt_string(form.pincode)),
        state: Set(opt_string(form.state)),
        gstin: Set(opt_string(form.gstin)),
        pan: Set(opt_string(form.pan)),
        phone: Set(opt_string(form.phone)),
        email: Set(opt_string(form.email)),
        website: Set(opt_string(form.website)),
    };
    match model.insert(&state.db).await {
        Ok(saved) => Redirect::to(&CustomerDetailRouteTag::new(saved.id).url()).into_response(),
        Err(_) => Redirect::to("/finance-customers/create/").into_response(),
    }
}

pub async fn edit_get(
    Cap(state): Cap<CustomerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-customers/").into_response();
    }
    let Some(customer) = find_customer_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/finance-customers/").into_response();
    };
    let (address_line_1, address_line_2, city, pincode, state) = customer_address_fields(&customer);
    let page = CustomerFormPage {
        id: customer.id,
        name: customer.name,
        address_line_1,
        address_line_2,
        city,
        pincode,
        state,
        gstin: customer.gstin.unwrap_or_default(),
        pan: customer.pan.unwrap_or_default(),
        phone: customer.phone.unwrap_or_default(),
        email: customer.email.unwrap_or_default(),
        website: customer.website.unwrap_or_default(),
        is_edit: true,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<CustomerState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Form(form): Form<CustomerForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-customers/").into_response();
    }
    let Some(existing) = find_customer_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/finance-customers/").into_response();
    };
    let now = Utc::now();
    let model = customer::ActiveModel {
        id: Set(existing.id),
        updated_at: Set(Some(now)),
        name: Set(form.name),
        address_line_1: Set(opt_string(form.address_line_1)),
        address_line_2: Set(opt_string(form.address_line_2)),
        city: Set(opt_string(form.city)),
        pincode: Set(opt_string(form.pincode)),
        state: Set(opt_string(form.state)),
        gstin: Set(opt_string(form.gstin)),
        pan: Set(opt_string(form.pan)),
        phone: Set(opt_string(form.phone)),
        email: Set(opt_string(form.email)),
        website: Set(opt_string(form.website)),
        ..Default::default()
    };
    if model.update(&state.db).await.is_ok() {
        Redirect::to(&CustomerDetailRouteTag::new(id).url()).into_response()
    } else {
        Redirect::to(&CustomerEditGetRouteTag::new(id).url()).into_response()
    }
}

pub async fn delete_post(
    Cap(state): Cap<CustomerState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-customers/").into_response();
    }
    if let Some(existing) = find_customer_scoped(&state.db, id, &ctx).await {
        let now = Utc::now();
        let model = customer::ActiveModel {
            id: Set(existing.id),
            deleted_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        let _ = model.update(&state.db).await;
    }
    Redirect::to("/finance-customers/").into_response()
}

pub async fn select(
    Cap(state): Cap<CustomerState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<CustomerSelectQuery>,
) -> maud::Markup {
    let customers = load_customer_rows(&state.db, &q.filter, &ctx, PAGE_SIZE).await;
    let page = CustomerSelectPage {
        customers,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_email: q.filter.email.clone().unwrap_or_default(),
        target_input: q
            .target_input
            .clone()
            .unwrap_or_else(|| "CustomerID".into()),
        path_and_query: path_and_query(&uri),
    };
    respond_picker_select::<CustomerSelectTableKey, CustomerSelectModalKey, _>(&htmx, &page)
}
