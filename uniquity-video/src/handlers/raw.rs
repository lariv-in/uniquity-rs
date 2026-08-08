use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use serde::Deserialize;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done},
    template::RenderAppPane,
};
use uniquity_employees::{
    handlers::employees::{EmployeeListQuery, EmployeeSelectQuery},
    scope::{employee_display_name, query_employees},
};

use crate::{
    entities::raw_footage::{self, Column as RawFootageColumn},
    forms::RawFootageForm,
    keys::{
        RawCreateModalKey, RawFootageSelectTableKey, RawFootageTableKey, VideoEmployeeSelectTableKey,
    },
    routes::{
        RawDetailRouteTag, RawEditGetRouteTag,
    },
    scope::{
        RawFootageRow, find_raw_footage, load_vnode_names, query_raw_footages, scope_raw_select,
        sync_raw_footage_files,
    },
    state::VideoState,
    templates::{
        RawCreateModalPage, RawDetailPage, RawFormPage, RawListPage, RawSelectPage,
        VideoEmployeeSelectPage,
    },
};

use super::ModalNameQuery;

#[derive(Debug, Deserialize, Default)]
pub struct RawListQuery {
    #[serde(default, rename = "Title", alias = "title")]
    pub title: Option<String>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RawSelectQuery {
    #[serde(default, rename = "Title", alias = "title")]
    pub title: Option<String>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

async fn raw_file_items(db: &sea_orm::DatabaseConnection, file_ids: &[i64]) -> Vec<ManyToManyItem> {
    let names = load_vnode_names(db, file_ids).await;
    file_ids
        .iter()
        .filter_map(|id| {
            names.get(id).map(|name| ManyToManyItem {
                key: id.to_string(),
                value: name.clone(),
            })
        })
        .collect()
}

async fn load_rows(
    db: &sea_orm::DatabaseConnection,
    auth: &lariv_rs::plugins::users::state::AuthContext,
    q: &RawListQuery,
) -> ObjectList<RawFootageRow> {
    let (rows, page, total) = query_raw_footages(
        db,
        auth,
        q.title.as_deref(),
        q.page.unwrap_or(1),
        q.sort.as_deref(),
    )
    .await;
    ObjectList::from_page(rows, page, DEFAULT_PAGE_SIZE, total)
}

pub async fn list(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<RawListQuery>,
) -> maud::Markup {
    let items = load_rows(&state.db, &ctx, &q).await;
    let page = RawListPage {
        items,
        filter_title: q.title.clone().unwrap_or_default(),
        sort: q.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<RawFootageTableKey>() {
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
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(raw) = find_raw_footage(&state.db, id).await else {
        return Redirect::to("/video/raw/").into_response();
    };
    let page = RawDetailPage {
        id: raw.id,
        title: raw.title,
        assigned_to_name: raw.assigned_to_name,
        file_names: raw.file_names,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    let page = RawCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        title: String::new(),
        assigned_to_id: 0,
        assigned_display: String::new(),
        file_items: vec![],
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    HtmlFormBody(form): HtmlFormBody<RawFootageForm>,
) -> Response {
    let now = Utc::now();
    let model = raw_footage::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        title: Set(form.title.clone()),
        assigned_to_id: Set(form.assigned_to_id),
    };
    match model.insert(&state.db).await {
        Ok(saved) => {
            if sync_raw_footage_files(&state.db, saved.id, &form.files)
                .await
                .is_err()
            {
                let page = RawCreateModalPage {
                    form_name: q.form_name(),
                    refresh_table: q.refresh_table(),
                    title: form.title,
                    assigned_to_id: form.assigned_to_id,
                    assigned_display: employee_display_name(&state.db, form.assigned_to_id).await,
                    file_items: raw_file_items(&state.db, &form.files).await,
                    error: "Failed to attach files".into(),
                };
                return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&_ctx))
                    .into_response();
            }
            respond_create_modal_done::<RawCreateModalKey>(
                &htmx,
                &q.refresh_table(),
                &RawDetailRouteTag::new(saved.id).url(),
            )
        }
        Err(e) => {
            let page = RawCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                title: form.title,
                assigned_to_id: form.assigned_to_id,
                assigned_display: employee_display_name(&state.db, form.assigned_to_id).await,
                file_items: raw_file_items(&state.db, &form.files).await,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&_ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(raw) = find_raw_footage(&state.db, id).await else {
        return Redirect::to("/video/raw/").into_response();
    };
    let file_items: Vec<ManyToManyItem> = raw
        .file_ids
        .iter()
        .zip(raw.file_names.iter())
        .map(|(id, name)| ManyToManyItem {
            key: id.to_string(),
            value: name.clone(),
        })
        .collect();
    let page = RawFormPage {
        id: raw.id,
        title: raw.title,
        assigned_to_id: raw.assigned_to_id,
        assigned_display: raw.assigned_to_name,
        file_items,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<VideoState>,
    RequireAuth(_ctx): RequireAuth,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<RawFootageForm>,
) -> Response {
    let Some(existing) = find_raw_footage(&state.db, id).await else {
        return Redirect::to("/video/raw/").into_response();
    };
    let now = Utc::now();
    let model = raw_footage::ActiveModel {
        id: Set(existing.id),
        updated_at: Set(Some(now)),
        title: Set(form.title),
        assigned_to_id: Set(form.assigned_to_id),
        ..Default::default()
    };
    if model.update(&state.db).await.is_ok() {
        let _ = sync_raw_footage_files(&state.db, id, &form.files).await;
        Redirect::to(&RawDetailRouteTag::new(id).url()).into_response()
    } else {
        Redirect::to(&RawEditGetRouteTag::new(id).url()).into_response()
    }
}

pub async fn delete_post(
    Cap(state): Cap<VideoState>,
    RequireAuth(_ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if find_raw_footage(&state.db, id).await.is_some() {
        let _ = raw_footage::Entity::delete_by_id(id).exec(&state.db).await;
    }
    Redirect::to("/video/raw/").into_response()
}

pub async fn select(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<RawSelectQuery>,
) -> maud::Markup {
    let mut query = crate::entities::RawFootageEntity::find();
    query = scope_raw_select(query, &state.db, &ctx).await;
    if let Some(t) = q.title.as_deref().filter(|s| !s.is_empty()) {
        query = query.filter(RawFootageColumn::Title.contains(t));
    }
    let sort = q.sort.as_deref().unwrap_or("").trim();
    query = match sort {
        s if s.eq_ignore_ascii_case("Title DESC") => query.order_by_desc(RawFootageColumn::Title),
        s if s.eq_ignore_ascii_case("Title ASC") || s.eq_ignore_ascii_case("Title") => {
            query.order_by_asc(RawFootageColumn::Title)
        }
        _ => query.order_by_desc(RawFootageColumn::UpdatedAt),
    };
    let page_num = q.page.unwrap_or(1).max(1);
    let paginator = query.paginate(&state.db, DEFAULT_PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let rows: Vec<RawFootageRow> = models
        .into_iter()
        .map(|m| RawFootageRow {
            id: m.id,
            title: m.title,
            assigned_to_name: String::new(),
        })
        .collect();
    let items = ObjectList::from_page(rows, page_num, DEFAULT_PAGE_SIZE, total);
    let page = RawSelectPage {
        items,
        filter_title: q.title.clone().unwrap_or_default(),
        sort: q.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
        target_input: q
            .target_input
            .clone()
            .unwrap_or_else(|| "RawFootageID".into()),
    };
    if htmx.targets::<RawFootageSelectTableKey>() {
        return page.render_table();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn employee_select(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<EmployeeSelectQuery>,
) -> maud::Markup {
    let list_q = EmployeeListQuery {
        name: q.name.clone(),
        email: q.email.clone(),
        page: q.page,
    };
    let (rows, page, total) = query_employees(
        &state.db,
        &ctx,
        list_q.name.as_deref(),
        list_q.email.as_deref(),
        list_q.page.unwrap_or(1),
        20,
    )
    .await;
    let employees = ObjectList::from_page(rows, page, DEFAULT_PAGE_SIZE, total);
    let page = VideoEmployeeSelectPage {
        employees,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_email: q.email.clone().unwrap_or_default(),
        target_input: q
            .target_input
            .clone()
            .unwrap_or_else(|| "AssignedToID".into()),
    };
    if htmx.targets::<VideoEmployeeSelectTableKey>() {
        return page.render_table();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}
