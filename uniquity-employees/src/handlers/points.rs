use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde::Deserialize;
use std::str::FromStr;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::{
        middleware::RequireAuth,
        state::AuthContext,
    },
    web::{
        Htmx, html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done,
    },
    template::RenderAppPane,
};

use uniquity_common::require_superuser;

use crate::{
    entities::points_transaction,
    forms::PointsForm,
    keys::{PointsCreateModalKey, PointsTableKey},
    routes::PointsDetailRouteTag,
    scope::{PointsRow, employee_display_name, find_points_scoped, query_points},
    state::EmployeesState,
    templates::{PointsCreateModalPage, PointsDetailPage, PointsListPage},
};

use super::ModalNameQuery;

const PAGE_SIZE: u64 = DEFAULT_PAGE_SIZE as u64;

#[derive(Debug, Deserialize, Default)]
pub struct PointsListQuery {
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

async fn load_rows(
    db: &sea_orm::DatabaseConnection,
    q: &PointsListQuery,
    auth: &AuthContext,
) -> ObjectList<PointsRow> {
    let (rows, page, total) =
        query_points(db, auth, q.page.unwrap_or(1), PAGE_SIZE, q.sort.as_deref()).await;
    ObjectList::from_page(rows, page, PAGE_SIZE as u32, total)
}

pub async fn list(
    Cap(state): Cap<EmployeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PointsListQuery>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/").into_response();
    }
    let points = load_rows(&state.db, &q, &ctx).await;
    let page = PointsListPage {
        points,
        sort: q.sort.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<PointsTableKey>() {
        return page.render_table().into_response();
    }
    if htmx.wants_main_content() {
        return page.render_main().into_markup().into_response();
    }
    if htmx.wants_app_layout() {
        return page.render_pane().into_markup().into_response();
    }
    html_built_page_with_slots(&page, &chrome, &slot_ctx).into_response()
}

pub async fn detail(
    Cap(state): Cap<EmployeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/points/").into_response();
    }
    let Some(pt) = find_points_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/employees/points/").into_response();
    };
    let rows = query_points(&state.db, &ctx, 1, 1000, None).await.0;
    let detail = rows.into_iter().find(|r| r.id == pt.id).unwrap_or(PointsRow {
        id: pt.id,
        points: pt.points,
        from_user_name: "—".into(),
        to_employee_name: "—".into(),
        created_at: pt
            .created_at
            .map(|t| t.format("%d/%m/%Y %H:%M").to_string())
            .unwrap_or_default(),
    });
    let page = PointsDetailPage {
        id: detail.id,
        points: detail.points.to_string(),
        from_user_name: detail.from_user_name,
        to_employee_name: detail.to_employee_name,
        created_at: detail.created_at,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/points/").into_response();
    }
    let page = PointsCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        to_employee_id: 0,
        employee_display: String::new(),
        points: String::new(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<EmployeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    Form(form): Form<PointsForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/points/").into_response();
    }
    let points = match Decimal::from_str(form.points.trim()) {
        Ok(d) => d,
        Err(_) => {
            let page = PointsCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                to_employee_id: form.to_employee_id,
                employee_display: employee_display_name(&state.db, form.to_employee_id).await,
                points: form.points,
                error: "Invalid points value".into(),
            };
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    match create_for_employee(&state.db, &ctx, form.to_employee_id, points).await {
        Ok(saved) => respond_create_modal_done::<PointsCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &PointsDetailRouteTag::new(saved.id).url(),
        ),
        Err(e) => {
            let page = PointsCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                to_employee_id: form.to_employee_id,
                employee_display: employee_display_name(&state.db, form.to_employee_id).await,
                points: form.points,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

/// Create a points transaction for a given employee (used by video editor-points route).
pub async fn create_for_employee(
    db: &sea_orm::DatabaseConnection,
    auth: &AuthContext,
    to_employee_id: i64,
    points: Decimal,
) -> Result<points_transaction::Model, sea_orm::DbErr> {
    if !auth.user.is_superuser {
        return Err(sea_orm::DbErr::Custom(
            "only superusers can create points transactions".into(),
        ));
    }
    let now = Utc::now();
    points_transaction::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        points: Set(points),
        from_user_id: Set(auth.user.id),
        to_employee_id: Set(to_employee_id),
    }
    .insert(db)
    .await
}
