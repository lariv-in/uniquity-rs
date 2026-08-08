use axum::{
    Form,
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};
use serde::Deserialize;

use lariv_rs::{
    components::{DEFAULT_PAGE_SIZE, ObjectList, SharedChromeFolder, SlotCtx},
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    template::RenderAppPane,
    web::{
        Htmx, html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done,
    },
};

use uniquity_common::require_superuser;

use crate::{
    entities::payment_term::{self, Entity as PaymentTermEntity, PAYMENT_TERM_TYPE_DUE_DATE, PAYMENT_TERM_TYPE_RELATIVE},
    forms::PaymentTermForm,
    keys::{
        PaymentTermCreateModalKey, PaymentTermSelectModalKey, PaymentTermSelectTableKey,
        PaymentTermTableKey,
    },
    logic::{
        create_payment_term, parse_due_datetime, payment_term_form_values, payment_term_summary,
        update_payment_term, CreatePaymentTermDueDate, CreatePaymentTermInput, CreatePaymentTermRelative,
    },
    routes::{PaymentTermDetailRouteTag, PaymentTermEditGetRouteTag},
    state::InvoicesState,
    templates::{
        PaymentTermCreateModalPage, PaymentTermDetailPage, PaymentTermFormPage,
        PaymentTermListPage, PaymentTermRow, PaymentTermSelectPage,
    },
};

use super::ModalNameQuery;

const PAGE_SIZE: u32 = DEFAULT_PAGE_SIZE;

#[derive(Debug, serde::Deserialize, Default)]
pub struct ListQuery {
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PaymentTermSelectQuery {
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

pub async fn list(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<ListQuery>,
) -> maud::Markup {
    let page_num = q.page.unwrap_or(1).max(1);
    let query = PaymentTermEntity::find()
        .order_by_desc(payment_term::Column::Id);
    let paginator = query.paginate(&state.db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for pt in models {
        rows.push(PaymentTermRow {
            id: pt.id,
            term_type: pt.term_type.clone(),
            summary: payment_term_summary(&state.db, &pt, &ctx.timezone).await,
        });
    }
    let terms = ObjectList::from_page(rows, page_num, PAGE_SIZE, total);
    let page = PaymentTermListPage {
        terms,
        path_and_query: path_and_query(&uri),
        can_edit: require_superuser(&ctx),
    };
    let slot_ctx = SlotCtx::from_auth(&ctx);
    if htmx.targets::<PaymentTermTableKey>() {
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

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalNameQuery>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/payment-terms/").into_response();
    }
    let page = PaymentTermCreateModalPage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        form: PaymentTermForm {
            term_type: PAYMENT_TERM_TYPE_DUE_DATE.to_string(),
            due_datetime: String::new(),
            duration: String::new(),
        },
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalNameQuery>,
    Form(form): Form<PaymentTermForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/payment-terms/").into_response();
    }
    let input = match form.term_type.as_str() {
        PAYMENT_TERM_TYPE_DUE_DATE => {
            let dt = match parse_due_datetime(&form.due_datetime, &ctx.timezone) {
                Ok(d) => d,
                Err(e) => {
                    let page = PaymentTermCreateModalPage {
                        form_name: q.form_name(),
                        refresh_table: q.refresh_table(),
                        form,
                        error: e.to_string(),
                    };
                    return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                        .into_response();
                }
            };
            CreatePaymentTermInput::DueDate(CreatePaymentTermDueDate { datetime: dt })
        }
        PAYMENT_TERM_TYPE_RELATIVE => {
            let nanos = match lariv_rs::duration::parse_duration(&form.duration) {
                Ok(n) => n,
                Err(e) => {
                    let page = PaymentTermCreateModalPage {
                        form_name: q.form_name(),
                        refresh_table: q.refresh_table(),
                        form,
                        error: e.to_string(),
                    };
                    return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                        .into_response();
                }
            };
            CreatePaymentTermInput::Relative(CreatePaymentTermRelative {
                duration_nanos: nanos,
            })
        }
        _ => {
            let page = PaymentTermCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                form,
                error: "Invalid payment term type".into(),
            };
            return html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response();
        }
    };
    match create_payment_term(&state.db, input).await {
        Ok(pt) => respond_create_modal_done::<PaymentTermCreateModalKey>(
            &htmx,
            &q.refresh_table(),
            &PaymentTermDetailRouteTag::new(pt.id).url(),
        ),
        Err(e) => {
            let page = PaymentTermCreateModalPage {
                form_name: q.form_name(),
                refresh_table: q.refresh_table(),
                form,
                error: e.to_string(),
            };
            html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
        }
    }
}

pub async fn detail(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(pt) = PaymentTermEntity::find_by_id(id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return Redirect::to("/finance-invoices/payment-terms/").into_response();
    };
    let summary = payment_term_summary(&state.db, &pt, &ctx.timezone).await;
    let page = PaymentTermDetailPage {
        id: pt.id,
        term_type: pt.term_type,
        summary,
        can_edit: require_superuser(&ctx),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_get(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/payment-terms/").into_response();
    }
    let Some(pt) = PaymentTermEntity::find_by_id(id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return Redirect::to("/finance-invoices/payment-terms/").into_response();
    };
    let summary = payment_term_summary(&state.db, &pt, &ctx.timezone).await;
    let values = payment_term_form_values(&state.db, &pt, &ctx.timezone).await;
    let page = PaymentTermFormPage {
        id: pt.id,
        form: PaymentTermForm {
            term_type: values.term_type,
            due_datetime: values.due_datetime,
            duration: values.duration,
        },
        summary,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Form(form): Form<PaymentTermForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/payment-terms/").into_response();
    }
    let input = match form.term_type.as_str() {
        PAYMENT_TERM_TYPE_DUE_DATE => {
            let dt = match parse_due_datetime(&form.due_datetime, &ctx.timezone) {
                Ok(d) => d,
                Err(_) => return Redirect::to(&PaymentTermEditGetRouteTag::new(id).url()).into_response(),
            };
            CreatePaymentTermInput::DueDate(CreatePaymentTermDueDate { datetime: dt })
        }
        PAYMENT_TERM_TYPE_RELATIVE => {
            let nanos = match lariv_rs::duration::parse_duration(&form.duration) {
                Ok(n) => n,
                Err(_) => return Redirect::to(&PaymentTermEditGetRouteTag::new(id).url()).into_response(),
            };
            CreatePaymentTermInput::Relative(CreatePaymentTermRelative {
                duration_nanos: nanos,
            })
        }
        _ => return Redirect::to(&PaymentTermEditGetRouteTag::new(id).url()).into_response(),
    };
    match update_payment_term(&state.db, id, input).await {
        Ok(_) => Redirect::to(&PaymentTermDetailRouteTag::new(id).url()).into_response(),
        Err(_) => Redirect::to(&PaymentTermEditGetRouteTag::new(id).url()).into_response(),
    }
}

pub async fn delete_post(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/payment-terms/").into_response();
    }
    if PaymentTermEntity::find_by_id(id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        let _ = PaymentTermEntity::delete_by_id(id).exec(&state.db).await;
    }
    Redirect::to("/finance-invoices/payment-terms/").into_response()
}

pub async fn fk_select(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<PaymentTermSelectQuery>,
) -> maud::Markup {
    let page_num = q.page.unwrap_or(1).max(1);
    let query = PaymentTermEntity::find()
        .order_by_desc(payment_term::Column::Id);
    let paginator = query.paginate(&state.db, PAGE_SIZE as u64);
    let total = paginator.num_items().await.unwrap_or(0);
    let models = paginator
        .fetch_page((page_num as u64).saturating_sub(1))
        .await
        .unwrap_or_default();
    let mut rows = Vec::with_capacity(models.len());
    for pt in models {
        rows.push(PaymentTermRow {
            id: pt.id,
            term_type: pt.term_type.clone(),
            summary: payment_term_summary(&state.db, &pt, &ctx.timezone).await,
        });
    }
    let terms = ObjectList::from_page(rows, page_num, PAGE_SIZE, total);
    let page = PaymentTermSelectPage {
        terms,
        target_input: q
            .target_input
            .unwrap_or_else(|| "PaymentTermID".into()),
        path_and_query: path_and_query(&uri),
        can_edit: require_superuser(&ctx),
    };
    respond_picker_select::<PaymentTermSelectTableKey, PaymentTermSelectModalKey, _>(&htmx, &page)
}
