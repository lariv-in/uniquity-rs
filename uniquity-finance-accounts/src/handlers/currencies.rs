use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait};
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
    entities::currency::{self, Entity as CurrencyEntity},
    forms::CurrencyForm,
    keys::{CurrencySelectModalKey, CurrencySelectTableKey, CurrencyTableKey},
    routes::{CurrencyDetailRouteTag, CurrencyEditGetRouteTag, CurrencyListRouteTag},
    scope::{apply_currency_filters, find_currency_scoped, scope_superuser},
    state::AccountsState,
    templates::{
        CurrencyDetailPage, CurrencyFormPage, CurrencyListPage,
        CurrencyRow, CurrencySelectPage,
    },
};

use super::util::{parse_i32, path_and_query};

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;

#[derive(Debug, Deserialize, Default)]
pub struct CurrencyListQuery {
    #[serde(default, rename = "Code", alias = "code")]
    pub code: Option<String>,
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "Symbol", alias = "symbol")]
    pub symbol: Option<String>,
    #[serde(default, rename = "MinorUnit", alias = "minor_unit")]
    pub minor_unit: Option<String>,
    #[serde(default)]
    pub page: QueryPage,
}

#[derive(Debug, Deserialize, Default)]
pub struct CurrencySelectQuery {
    #[serde(flatten)]
    pub filter: CurrencyListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

async fn load_currency_rows(
    db: &sea_orm::DatabaseConnection,
    q: &CurrencyListQuery,
    auth: &AuthContext,
) -> ObjectList<CurrencyRow> {
    let mut query = CurrencyEntity::find();
    query = apply_currency_filters(
        query,
        q.code.as_deref(),
        q.name.as_deref(),
        q.symbol.as_deref(),
        q.minor_unit.as_deref(),
    );
    query = scope_superuser(query, auth);
    let page = q.page.get();
    let paginator = query.paginate(db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows: Vec<CurrencyRow> = models
        .into_iter()
        .map(|c| CurrencyRow {
            id: c.id,
            code: c.code,
            name: c.name,
            symbol: c.symbol,
            minor_unit: c.minor_unit,
        })
        .collect();
    ObjectList::from_page(rows, page, PAGE_SIZE, total)
}

pub async fn list(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<CurrencyListQuery>,
) -> maud::Markup {
    let currencies = load_currency_rows(&state.db, &q, &ctx).await;
    let page = CurrencyListPage {
        currencies,
        filter_code: q.code.clone().unwrap_or_default(),
        filter_name: q.name.clone().unwrap_or_default(),
        filter_symbol: q.symbol.clone().unwrap_or_default(),
        filter_minor_unit: q.minor_unit.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        can_edit: require_superuser(&ctx),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<CurrencyTableKey>() {
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
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(c) = find_currency_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    };
    let page = CurrencyDetailPage {
        id: c.id,
        code: c.code,
        name: c.name,
        symbol: c.symbol,
        minor_unit: c.minor_unit,
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
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    }
    let page = CurrencyFormPage::new(false);
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    Form(form): Form<CurrencyForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    }
    let now = Utc::now();
    let model = currency::ActiveModel {
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        code: Set(parse_i32(&form.code).unwrap_or(0)),
        name: Set(form.name),
        symbol: Set(form.symbol),
        minor_unit: Set(parse_i32(&form.minor_unit).unwrap_or(0)),
        ..Default::default()
    };
    match model.insert(&state.db).await {
        Ok(saved) => Redirect::to(&CurrencyDetailRouteTag::new(saved.id).url()).into_response(),
        Err(_) => Redirect::to("/finance/currencies/create").into_response(),
    }
}

pub async fn edit_get(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    }
    let Some(c) = find_currency_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    };
    let page = CurrencyFormPage::from_model(&c, true);
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Form(form): Form<CurrencyForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    }
    let Some(existing) = find_currency_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    };
    let now = Utc::now();
    let model = currency::ActiveModel {
        id: Set(existing.id),
        updated_at: Set(Some(now)),
        code: Set(parse_i32(&form.code).unwrap_or(existing.code)),
        name: Set(form.name),
        symbol: Set(form.symbol),
        minor_unit: Set(parse_i32(&form.minor_unit).unwrap_or(existing.minor_unit)),
        ..Default::default()
    };
    if model.update(&state.db).await.is_ok() {
        Redirect::to(&CurrencyDetailRouteTag::new(id).url()).into_response()
    } else {
        Redirect::to(&CurrencyEditGetRouteTag::new(id).url()).into_response()
    }
}

pub async fn delete_post(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    }
    let Some(existing) = find_currency_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(&CurrencyListRouteTag.url()).into_response();
    };
    let now = Utc::now();
    let model = currency::ActiveModel {
        id: Set(existing.id),
        deleted_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    let _ = model.update(&state.db).await;
    Redirect::to(&CurrencyListRouteTag.url()).into_response()
}

pub async fn select(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<CurrencySelectQuery>,
) -> maud::Markup {
    let currencies = load_currency_rows(&state.db, &q.filter, &ctx).await;
    let page = CurrencySelectPage {
        currencies,
        filter_code: q.filter.code.clone().unwrap_or_default(),
        filter_name: q.filter.name.clone().unwrap_or_default(),
        filter_symbol: q.filter.symbol.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.unwrap_or_else(|| "CurrencyId".into()),
    };
    respond_picker_select::<CurrencySelectTableKey, CurrencySelectModalKey, _>(&htmx, &page)
}
