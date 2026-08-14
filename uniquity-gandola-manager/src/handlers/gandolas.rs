use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::{NaiveDate, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait, QueryOrder};

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    picker::respond_picker_select,
    plugins::users::{middleware::RequireAuth, state::AuthContext},
    template::RenderAppPane,
    web::{
        Htmx, QueryPage, html_built_page_or_app_layout, html_built_page_with_slots,
        respond_create_modal_done_fk, respond_edit_modal_done,
    },
};

use crate::{
    entities::gandola::{self, Entity as GandolaEntity},
    forms::GandolaForm,
    handlers::ModalNameQuery,
    keys::{
        GandolaCreateModalKey, GandolaEditModalKey, GandolaSelectModalKey, GandolaSelectTableKey,
        GandolaTableKey,
    },
    logic::current_site_for,
    routes::GandolaDetailRouteTag,
    scope::{
        apply_name_filter_gandolas, find_gandola_scoped, is_superuser, load_sites_for_gandola,
        scope_gandolas, site_items_for_gandola, site_items_from_ids, sync_gandola_sites,
    },
    state::GandolaManagerState,
    templates::{
        GandolaCreateModalPage, GandolaDetailPage, GandolaEditModalPage, GandolaListPage,
        GandolaRow, GandolaSelectPage, RelatedName,
    },
};

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;
const LIST_URL: &str = "/gandola/";

#[derive(Debug, serde::Deserialize, Default)]
pub struct GandolaListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: QueryPage,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct GandolaSelectQuery {
    #[serde(flatten)]
    pub filter: GandolaListQuery,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

fn today_utc() -> NaiveDate {
    Utc::now().date_naive()
}

async fn gandola_to_row(db: &sea_orm::DatabaseConnection, g: gandola::Model) -> GandolaRow {
    let sites = load_sites_for_gandola(db, g.id).await;
    let current = current_site_for(&sites, today_utc());
    GandolaRow {
        id: g.id,
        name: g.name,
        is_assigned: current.is_some(),
        current_site_name: current.map(|s| s.name.clone()).unwrap_or_default(),
        site_names: sites.iter().map(|s| s.name.clone()).collect(),
    }
}

async fn query_gandolas(
    db: &sea_orm::DatabaseConnection,
    q: &GandolaListQuery,
    auth: &AuthContext,
    page_size: u32,
) -> ObjectList<GandolaRow> {
    let mut query = GandolaEntity::find();
    query = apply_name_filter_gandolas(query, q.name.as_deref());
    query = scope_gandolas(query, auth);
    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Name DESC") => query.order_by_desc(gandola::Column::Name),
        s if s.eq_ignore_ascii_case("Name ASC") || s.eq_ignore_ascii_case("Name") => {
            query.order_by_asc(gandola::Column::Name)
        }
        _ => query
            .order_by_desc(gandola::Column::CreatedAt)
            .order_by_desc(gandola::Column::Id),
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
        rows.push(gandola_to_row(db, model).await);
    }
    ObjectList::from_page(rows, page, page_size, total)
}

pub async fn list(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<GandolaListQuery>,
) -> maud::Markup {
    let gandolas = query_gandolas(&state.db, &q, &ctx, PAGE_SIZE).await;
    let page = GandolaListPage {
        gandolas,
        filter_name: q.name.clone().unwrap_or_default(),
        sort: q.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        can_edit: is_superuser(&ctx),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<GandolaTableKey>() {
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
    let Some(g) = find_gandola_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(LIST_URL).into_response();
    };
    let sites = load_sites_for_gandola(&state.db, g.id).await;
    let current = current_site_for(&sites, today_utc());
    let page = GandolaDetailPage {
        id: g.id,
        name: g.name,
        is_assigned: current.is_some(),
        current_site: current.map(|s| RelatedName {
            id: s.id,
            name: s.name.clone(),
        }),
        sites: sites
            .into_iter()
            .map(|s| RelatedName {
                id: s.id,
                name: s.name,
            })
            .collect(),
        can_edit: is_superuser(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

fn create_modal_from_form(
    form: &GandolaForm,
    sites: Vec<ManyToManyItem>,
    form_name: String,
    refresh_table: String,
    target_input: String,
    error: String,
) -> GandolaCreateModalPage {
    GandolaCreateModalPage {
        form_name,
        refresh_table,
        target_input,
        name: form.name.clone(),
        sites,
        error,
    }
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let page = GandolaCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        name: String::new(),
        sites: Vec::new(),
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
    Form(form): Form<GandolaForm>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let sites = site_items_from_ids(&state.db, &form.sites).await;
    let render_error = |error: String| {
        let page = create_modal_from_form(
            &form,
            sites.clone(),
            q.form_name(),
            q.refresh_table(),
            q.target_input(),
            error,
        );
        html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
    };
    if form.name.trim().is_empty() {
        return render_error("Name is required".into());
    }
    let now = Utc::now();
    let model = gandola::ActiveModel {
        name: Set(form.name.trim().to_string()),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            if let Err(e) = sync_gandola_sites(&state.db, saved.id, &form.sites).await {
                return render_error(e);
            }
            respond_create_modal_done_fk::<GandolaCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &GandolaDetailRouteTag::new(saved.id).url(),
                saved.id,
                &saved.name,
                &q.target_input(),
            )
        }
        Err(e) => render_error(e.to_string()),
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
    let Some(g) = find_gandola_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(LIST_URL).into_response();
    };
    let page = GandolaEditModalPage {
        id: g.id,
        form_name: q.form_name(),
        name: g.name,
        sites: site_items_for_gandola(&state.db, g.id).await,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(q): Query<ModalNameQuery>,
    Form(form): Form<GandolaForm>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let Some(existing) = find_gandola_scoped(&state.db, id, &ctx).await else {
        return Redirect::to(LIST_URL).into_response();
    };
    let sites = site_items_from_ids(&state.db, &form.sites).await;
    let render_error = |error: String| {
        let page = GandolaEditModalPage {
            id,
            form_name: q.form_name(),
            name: form.name.clone(),
            sites: sites.clone(),
            error,
        };
        html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
    };
    if form.name.trim().is_empty() {
        return render_error("Name is required".into());
    }
    let now = Utc::now();
    let model = gandola::ActiveModel {
        id: Set(existing.id),
        name: Set(form.name.trim().to_string()),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(_) => {
            if let Err(e) = sync_gandola_sites(&state.db, id, &form.sites).await {
                return render_error(e);
            }
            respond_edit_modal_done::<GandolaEditModalKey>(
                &htmx,
                &GandolaDetailRouteTag::new(id).url(),
            )
        }
        Err(e) => render_error(e.to_string()),
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
    if find_gandola_scoped(&state.db, id, &ctx).await.is_some() {
        let _ = GandolaEntity::delete_by_id(id).exec(&state.db).await;
    }
    Redirect::to(LIST_URL).into_response()
}

pub async fn select(
    Cap(state): Cap<GandolaManagerState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<GandolaSelectQuery>,
) -> maud::Markup {
    let gandolas = query_gandolas(&state.db, &q.filter, &ctx, PAGE_SIZE).await;
    let page = GandolaSelectPage {
        gandolas,
        filter_name: q.filter.name.clone().unwrap_or_default(),
        sort: q.filter.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q.target_input.clone().unwrap_or_else(|| "Gandolas".into()),
        can_edit: is_superuser(&ctx),
    };
    respond_picker_select::<GandolaSelectTableKey, GandolaSelectModalKey, _>(&htmx, &page)
}
