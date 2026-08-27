use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::Deserialize;

use lariv_rs::{
    html_form::HtmlFormBody,
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx, SwapKey},
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
    entities::employee,
    forms::EmployeeForm,
    keys::{
        EmployeeCreateModalKey, EmployeeDeleteModalKey, EmployeeSelectTableKey, EmployeeTableKey,
    },
    routes::{
        EmployeesDetailRouteTag, EmployeesEditGetRouteTag,
    },
    scope::{
        EmployeeRow, employee_points_total, find_employee_scoped,
        query_employees, user_display_name,
    },
    state::EmployeesState,
    templates::{
        ConfirmDeletePage, EmployeeCreateModalPage, EmployeeDetailPage, EmployeeFormPage,
        EmployeeListPage, EmployeeSelectPage,
    },
};

use super::ModalNameQuery;

const PAGE_SIZE: u64 = DEFAULT_PAGE_SIZE as u64;

#[derive(Debug, Deserialize, Default)]
pub struct EmployeeListQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "Email", alias = "email")]
    pub email: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EmployeeSelectQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default, rename = "Email", alias = "email")]
    pub email: Option<String>,
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

async fn load_rows(
    db: &sea_orm::DatabaseConnection,
    q: &EmployeeListQuery,
    auth: &AuthContext,
) -> ObjectList<EmployeeRow> {
    let (rows, page, total) = query_employees(
        db,
        auth,
        q.name.as_deref(),
        q.email.as_deref(),
        q.page.unwrap_or(1),
        PAGE_SIZE,
    )
    .await;
    ObjectList::from_page(rows, page, PAGE_SIZE as u32, total)
}

pub async fn list(
    Cap(state): Cap<EmployeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<EmployeeListQuery>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/").into_response();
    }
    let employees = load_rows(&state.db, &q, &ctx).await;
    let page = EmployeeListPage {
        employees,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_email: q.email.clone().unwrap_or_default(),
        path_and_query: path_and_query(&uri),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<EmployeeTableKey>() {
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
        return Redirect::to("/employees/").into_response();
    }
    let Some(emp) = find_employee_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/employees/").into_response();
    };
    let user_name = user_display_name(&state.db, emp.user_id).await;
    let user_email = crate::scope::load_user_map(&state.db, &[emp.user_id])
        .await
        .get(&emp.user_id)
        .map(|u| u.email.clone())
        .unwrap_or_default();
    let total_points = employee_points_total(&state.db, emp.id).await;
    let page = EmployeeDetailPage {
        id: emp.id,
        user_name,
        user_email,
        total_points: total_points.to_string(),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/").into_response();
    }
    let page = EmployeeCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        user_id: 0,
        user_display: String::new(),
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
    HtmlFormBody(form): HtmlFormBody<EmployeeForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/").into_response();
    }
    let user_display = user_display_name(&state.db, form.user_id).await;
    let now = Utc::now();
    let model = employee::ActiveModel {
        id: Default::default(),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        user_id: Set(form.user_id),
    };
    match model.insert(&state.db).await {
        Ok(saved) => respond_create_modal_done::<EmployeeCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &EmployeesDetailRouteTag::new(saved.id).url(),
        ),
        Err(e) => {
            let page = EmployeeCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                user_id: form.user_id,
                user_display,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn edit_get(
    Cap(state): Cap<EmployeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/").into_response();
    }
    let Some(emp) = find_employee_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/employees/").into_response();
    };
    let user_display = user_display_name(&state.db, emp.user_id).await;
    let page = EmployeeFormPage {
        id: emp.id,
        user_id: emp.user_id,
        user_display,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<EmployeesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<EmployeeForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/").into_response();
    }
    let Some(existing) = find_employee_scoped(&state.db, id, &ctx).await else {
        return Redirect::to("/employees/").into_response();
    };
    let now = Utc::now();
    let model = employee::ActiveModel {
        id: Set(existing.id),
        updated_at: Set(Some(now)),
        user_id: Set(form.user_id),
        ..Default::default()
    };
    if model.update(&state.db).await.is_ok() {
        Redirect::to(&EmployeesDetailRouteTag::new(id).url()).into_response()
    } else {
        Redirect::to(&EmployeesEditGetRouteTag::new(id).url()).into_response()
    }
}

pub async fn delete_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
    Path(id): Path<i64>,
) -> maud::Markup {
    let page = ConfirmDeletePage {
        modal_uid: EmployeeDeleteModalKey::ID.to_string(),
        message: "Are you sure you want to delete this employee?".into(),
        form_name: q
            .name
            .clone()
            .unwrap_or_else(|| "p_uniquity_employees.EmployeeDeleteForm".into()),
        id,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn delete_post(
    Cap(state): Cap<EmployeesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/employees/").into_response();
    }
    if find_employee_scoped(&state.db, id, &ctx).await.is_none() {
        return Redirect::to("/employees/").into_response();
    }
    match employee::Entity::delete_by_id(id).exec(&state.db).await {
        Ok(_) => htmx.redirect("/employees/"),
        Err(e) => {
            tracing::error!(error = %e, id, "failed to delete employee");
            let page = ConfirmDeletePage {
                modal_uid: EmployeeDeleteModalKey::ID.to_string(),
                message: "Are you sure you want to delete this employee?".into(),
                form_name: "p_uniquity_employees.EmployeeDeleteForm".into(),
                id,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn select(
    Cap(state): Cap<EmployeesState>,
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
    let employees = load_rows(&state.db, &list_q, &ctx).await;
    let page = EmployeeSelectPage {
        employees,
        filter_name: q.name.clone().unwrap_or_default(),
        filter_email: q.email.clone().unwrap_or_default(),
        target_input: q
            .target_input
            .clone()
            .unwrap_or_else(|| "ToEmployeeID".into()),
    };
    if htmx.targets::<EmployeeSelectTableKey>() {
        return page.render_table();
    }
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}
