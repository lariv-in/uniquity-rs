use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde::Deserialize;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done},
    template::RenderAppPane,
};

use crate::{
    entities::edited_video,
    forms::EditedVideoForm,
    keys::{EditedCreateModalKey, EditedVideoSelectTableKey, EditedVideoTableKey},
    routes::{EditedDetailRouteTag, EditedEditGetRouteTag},
    scope::{
        find_edited_video, query_edited_videos, raw_footage_title, vnode_display_name,
    },
    state::VideoState,
    templates::{
        EditedCreateModalPage, EditedDetailPage, EditedFormPage, EditedListPage,
        EditedSelectPage,
    },
};

use super::ModalNameQuery;

#[derive(Debug, Deserialize, Default)]
pub struct EditedListQuery {
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EditedSelectQuery {
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

async fn edited_create_modal_page(
    db: &sea_orm::DatabaseConnection,
    q: &ModalNameQuery,
    form: EditedVideoForm,
    error: String,
) -> EditedCreateModalPage {
    let raw_display = if form.raw_footage_id > 0 {
        raw_footage_title(db, form.raw_footage_id).await
    } else {
        String::new()
    };
    EditedCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        raw_footage_id: form.raw_footage_id,
        raw_display,
        edited_v_node_id: form.edited_v_node_id,
        vnode_display: vnode_display_name(db, form.edited_v_node_id).await,
        error,
    }
}

pub async fn list(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<EditedListQuery>,
) -> maud::Markup {
    let (rows, page, total) = query_edited_videos(&state.db, q.page.unwrap_or(1)).await;
    let items = ObjectList::from_page(rows, page, DEFAULT_PAGE_SIZE, total);
    let page = EditedListPage {
        items,
        path_and_query: path_and_query(&uri),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<EditedVideoTableKey>() {
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
    let Some(ev) = find_edited_video(&state.db, id).await else {
        return Redirect::to("/video/edited/").into_response();
    };
    let page = EditedDetailPage {
        id: ev.id,
        raw_footage_id: ev.raw_footage_id,
        raw_title: ev.raw_title,
        assigned_to_id: ev.assigned_to_id,
        assigned_to_name: ev.assigned_to_name,
        raw_file_names: ev.raw_file_names,
        output_name: ev.output_name,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    let page = EditedCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        raw_footage_id: 0,
        raw_display: String::new(),
        edited_v_node_id: 0,
        vnode_display: String::new(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    Form(form): Form<EditedVideoForm>,
) -> Response {
    let now = Utc::now();
    let model = edited_video::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        deleted_at: Set(None),
        raw_footage_id: Set(form.raw_footage_id),
        edited_v_node_id: Set(form.edited_v_node_id),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done::<EditedCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &EditedDetailRouteTag::new(saved.id).url(),
        ),
        Err(e) => {
            let page = edited_create_modal_page(&state.db, &q, form, e.to_string()).await;
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
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
    let Some(ev) = find_edited_video(&state.db, id).await else {
        return Redirect::to("/video/edited/").into_response();
    };
    let page = EditedFormPage {
        id: ev.id,
        raw_footage_id: ev.raw_footage_id,
        raw_display: ev.raw_title,
        edited_v_node_id: ev.edited_v_node_id,
        vnode_display: ev.output_name,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<VideoState>,
    RequireAuth(_ctx): RequireAuth,
    Path(id): Path<i64>,
    Form(form): Form<EditedVideoForm>,
) -> Response {
    let Some(existing) = find_edited_video(&state.db, id).await else {
        return Redirect::to("/video/edited/").into_response();
    };
    let now = Utc::now();
    let model = edited_video::ActiveModel {
        id: Set(existing.id),
        updated_at: Set(Some(now)),
        raw_footage_id: Set(form.raw_footage_id),
        edited_v_node_id: Set(form.edited_v_node_id),
        ..Default::default()
    };
    if model.update(&state.db).await.is_ok() {
        Redirect::to(&EditedDetailRouteTag::new(id).url()).into_response()
    } else {
        Redirect::to(&EditedEditGetRouteTag::new(id).url()).into_response()
    }
}

pub async fn delete_post(
    Cap(state): Cap<VideoState>,
    RequireAuth(_ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if let Some(existing) = find_edited_video(&state.db, id).await {
        let now = Utc::now();
        let model = edited_video::ActiveModel {
            id: Set(existing.id),
            deleted_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        };
        let _ = model.update(&state.db).await;
    }
    Redirect::to("/video/edited/").into_response()
}

pub async fn select(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<EditedSelectQuery>,
) -> maud::Markup {
    let (rows, page, total) = query_edited_videos(&state.db, q.page.unwrap_or(1)).await;
    let items = ObjectList::from_page(rows, page, DEFAULT_PAGE_SIZE, total);
    let page = EditedSelectPage {
        items,
        target_input: q
            .target_input
            .clone()
            .unwrap_or_else(|| "EditedVideoID".into()),
    };
    if htmx.targets::<EditedVideoSelectTableKey>() {
        return page.render_table();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}
