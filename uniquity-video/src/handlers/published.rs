use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::Deserialize;
use std::str::FromStr;
use tracing::warn;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done},
    template::RenderAppPane,
};
use uniquity_common::require_superuser;
use uniquity_employees::{
    handlers::points::create_for_employee,
    routes::PointsDetailRouteTag,
};

use crate::{
    entities::published_video,
    forms::{EditorPointsForm, PublishedVideoForm},
    keys::{PublishedCreateModalKey, PublishedVideoSelectTableKey, PublishedVideoTableKey},
    routes::{
        PublishedDetailRouteTag, PublishedEditGetRouteTag, PublishedEditorPointsPostRouteTag,
    },
    scope::{edited_video_display, find_published_video, query_published_videos},
    state::VideoState,
    templates::{
        EditorPointsPage, PublishedCreateModalPage, PublishedDetailPage, PublishedFormPage,
        PublishedListPage, PublishedSelectPage,
    },
    youtube::{self, YouTubeSnippetMeta, dash_if_empty, fetch_youtube_snippet_meta},
};

use super::ModalNameQuery;

#[derive(Debug, Deserialize, Default)]
pub struct PublishedListQuery {
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PublishedSelectQuery {
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

async fn load_youtube_meta(state: &VideoState, video_id: &str) -> YouTubeSnippetMeta {
    let key = state.config.youtube_api_key();
    if key.is_empty() {
        return YouTubeSnippetMeta::default();
    }
    match fetch_youtube_snippet_meta(&state.http, key, video_id).await {
        Ok(m) => m,
        Err(e) => {
            warn!(video_id, error = %e, "youtube metadata fetch failed");
            YouTubeSnippetMeta::default()
        }
    }
}

pub async fn list(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PublishedListQuery>,
) -> maud::Markup {
    let (rows, page, total) = query_published_videos(&state.db, q.page.unwrap_or(1)).await;
    let items = ObjectList::from_page(rows, page, DEFAULT_PAGE_SIZE, total);
    let page = PublishedListPage {
        items,
        path_and_query: path_and_query(&uri),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<PublishedVideoTableKey>() {
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
    let Some(pv) = find_published_video(&state.db, id).await else {
        return Redirect::to("/video/published/").into_response();
    };
    let yt = load_youtube_meta(&state, &pv.youtube_id).await;
    let watch_url = youtube::youtube_watch_url(&pv.youtube_id).unwrap_or_default();
    let studio_url = youtube::youtube_studio_url(&pv.youtube_id).unwrap_or_default();
    let page = PublishedDetailPage {
        id: pv.id,
        youtube_id: pv.youtube_id,
        watch_url,
        studio_url,
        yt_title: dash_if_empty(&yt.title).to_string(),
        yt_published_at: dash_if_empty(&yt.published_at_display).to_string(),
        yt_upload_status: dash_if_empty(&yt.upload_status).to_string(),
        yt_view_count: dash_if_empty(&yt.view_count).to_string(),
        yt_like_count: dash_if_empty(&yt.like_count).to_string(),
        yt_comment_count: dash_if_empty(&yt.comment_count).to_string(),
        raw_title: pv.raw_title,
        assigned_to_id: pv.assigned_to_id,
        assigned_to_name: pv.assigned_to_name,
        can_award_points: require_superuser(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    let page = PublishedCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        edited_video_id: 0,
        edited_display: String::new(),
        you_tube_video_id: String::new(),
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
    Form(form): Form<PublishedVideoForm>,
) -> Response {
    let video_id = match youtube::clean_youtube_video_id(&form.you_tube_video_id) {
        Ok(id) => id,
        Err(e) => {
            let page = PublishedCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                edited_video_id: form.edited_video_id,
                edited_display: edited_video_display(&state.db, form.edited_video_id).await,
                you_tube_video_id: form.you_tube_video_id,
                error: e.to_string(),
            };
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    let now = Utc::now();
    let model = published_video::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        edited_video_id: Set(form.edited_video_id),
        you_tube_video_id: Set(video_id),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done::<PublishedCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &PublishedDetailRouteTag::new(saved.id).url(),
        ),
        Err(e) => {
            let page = PublishedCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                edited_video_id: form.edited_video_id,
                edited_display: edited_video_display(&state.db, form.edited_video_id).await,
                you_tube_video_id: form.you_tube_video_id,
                error: e.to_string(),
            };
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
    let Some(pv) = find_published_video(&state.db, id).await else {
        return Redirect::to("/video/published/").into_response();
    };
    let edited_display = edited_video_display(&state.db, pv.edited_video_id).await;
    let page = PublishedFormPage {
        id: pv.id,
        edited_video_id: pv.edited_video_id,
        edited_display,
        you_tube_video_id: pv.youtube_id,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<VideoState>,
    RequireAuth(_ctx): RequireAuth,
    Path(id): Path<i64>,
    Form(form): Form<PublishedVideoForm>,
) -> Response {
    let Some(existing) = find_published_video(&state.db, id).await else {
        return Redirect::to("/video/published/").into_response();
    };
    let video_id = match youtube::clean_youtube_video_id(&form.you_tube_video_id) {
        Ok(vid) => vid,
        Err(_) => return Redirect::to(&PublishedEditGetRouteTag::new(id).url()).into_response(),
    };
    let now = Utc::now();
    let model = published_video::ActiveModel {
        id: Set(existing.id),
        updated_at: Set(Some(now)),
        edited_video_id: Set(form.edited_video_id),
        you_tube_video_id: Set(video_id),
        ..Default::default()
    };
    if model.update(&state.db).await.is_ok() {
        Redirect::to(&PublishedDetailRouteTag::new(id).url()).into_response()
    } else {
        Redirect::to(&PublishedEditGetRouteTag::new(id).url()).into_response()
    }
}

pub async fn delete_post(
    Cap(state): Cap<VideoState>,
    RequireAuth(_ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if find_published_video(&state.db, id).await.is_some() {
        let _ = published_video::Entity::delete_by_id(id).exec(&state.db).await;
    }
    Redirect::to("/video/published/").into_response()
}

pub async fn select(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<PublishedSelectQuery>,
) -> maud::Markup {
    let (rows, page, total) = query_published_videos(&state.db, q.page.unwrap_or(1)).await;
    let items = ObjectList::from_page(rows, page, DEFAULT_PAGE_SIZE, total);
    let page = PublishedSelectPage {
        items,
        target_input: q
            .target_input
            .clone()
            .unwrap_or_else(|| "PublishedVideoID".into()),
    };
    if htmx.targets::<PublishedVideoSelectTableKey>() {
        return page.render_table();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn editor_points_get(
    Cap(state): Cap<VideoState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&PublishedDetailRouteTag::new(id).url()).into_response();
    }
    let Some(pv) = find_published_video(&state.db, id).await else {
        return Redirect::to("/video/published/").into_response();
    };
    let page = EditorPointsPage {
        published_id: pv.id,
        editor_name: pv.assigned_to_name,
        points: String::new(),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn editor_points_post(
    Cap(state): Cap<VideoState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Form(form): Form<EditorPointsForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&PublishedDetailRouteTag::new(id).url()).into_response();
    }
    let Some(pv) = find_published_video(&state.db, id).await else {
        return Redirect::to("/video/published/").into_response();
    };
    if pv.assigned_to_id == 0 {
        return Redirect::to(&PublishedEditorPointsPostRouteTag::new(id).url()).into_response();
    }
    let points = match Decimal::from_str(form.points.trim()) {
        Ok(d) => d,
        Err(_) => {
            return Redirect::to(&crate::routes::PublishedEditorPointsGetRouteTag::new(id).url())
                .into_response()
        }
    };
    match create_for_employee(&state.db, &ctx, pv.assigned_to_id, points).await {
        Ok(saved) => Redirect::to(&PointsDetailRouteTag::new(saved.id).url()).into_response(),
        Err(_) => Redirect::to(&crate::routes::PublishedEditorPointsGetRouteTag::new(id).url())
            .into_response(),
    }
}
