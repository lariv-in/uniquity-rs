use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use lariv_rs::{
    components::{ManyToManyItem, SharedChromeFolder, SlotCtx},
    html_form::HtmlFormBody,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout, html_built_page_with_slots},
};

use uniquity_common::require_superuser;
use uniquity_finance_customer::entities::customer::{self, Entity as CustomerEntity};
use uniquity_finance_taxes::scope::{load_taxes_by_ids, tax_label};

use crate::{
    entities::{
        draft_invoice::{self, Entity as DraftInvoiceEntity},
        payment_term::Entity as PaymentTermEntity,
    },
    forms::{DraftInvoiceForm, PAYMENT_TERM_MODE_DATE, PAYMENT_TERM_MODE_TERM},
    logic::{
        create_draft_invoice, optional_display, optional_trimmed_text, parse_due_date,
        parse_invoice_datetime, parse_lines_json, soft_delete_draft, update_draft_invoice,
        CreateDraftInput, PaymentTermSelection, UpdateDraftInput,
    },
    logic::invoice_line_editor::{
        default_lines_json, draft_invoice_line_display_rows, draft_lines_form_json,
        invoice_line_editor_preview_json,
    },
    logic::payment_term::payment_term_summary,
    logic::tax_assoc::load_draft_invoice_tax_ids,
    routes::DraftInvoiceDetailRouteTag,
    state::InvoicesState,
    templates::{DraftInvoiceDetailPage, DraftInvoiceFormPage},
};

#[derive(Debug, serde::Deserialize, Default)]
pub struct DeleteQuery {
    #[serde(default)]
    pub confirmed: Option<bool>,
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct DetailQuery {
    #[serde(default)]
    pub error: Option<String>,
}

fn form_to_input(form: &DraftInvoiceForm, tz: &str) -> Result<CreateDraftInput, String> {
    if form.customer_id <= 0 {
        return Err("select a customer".to_string());
    }
    let payment_term = match form.payment_term_mode.as_str() {
        PAYMENT_TERM_MODE_DATE => {
            let datetime = parse_due_date(&form.payment_due_date, tz)?;
            PaymentTermSelection::DueDate(datetime)
        }
        _ => {
            if form.payment_term_id <= 0 {
                return Err("select a payment term".to_string());
            }
            PaymentTermSelection::Existing(form.payment_term_id)
        }
    };
    let lines = parse_lines_json(&form.invoice_lines_json)?;
    Ok(CreateDraftInput {
        number: Some(form.number.clone()),
        reference: optional_trimmed_text(&form.reference),
        payment_reference: optional_trimmed_text(&form.payment_reference),
        bank_account: optional_trimmed_text(&form.bank_account),
        datetime: parse_invoice_datetime(&form.datetime, tz),
        customer_id: form.customer_id,
        payment_term,
        header_tax_ids: form.taxes.clone(),
        lines,
    })
}

struct DraftFormContext {
    customer_display: String,
    payment_term_display: String,
    tax_items: Vec<ManyToManyItem>,
    invoice_lines_preview: String,
}

async fn load_draft_form_context(
    db: &sea_orm::DatabaseConnection,
    customer_id: i64,
    payment_term_id: i64,
    tax_ids: &[i64],
    tz: &str,
) -> DraftFormContext {
    let customer_display = if customer_id > 0 {
        CustomerEntity::find_by_id(customer_id)
            .filter(customer::Column::DeletedAt.is_null())
            .one(db)
            .await
            .ok()
            .flatten()
            .map(|c| c.name)
            .unwrap_or_default()
    } else {
        String::new()
    };

    let payment_term_display = if payment_term_id > 0 {
        if let Ok(Some(pt)) = PaymentTermEntity::find_by_id(payment_term_id).one(db).await {
            payment_term_summary(db, &pt, tz).await
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let taxes = load_taxes_by_ids(db, tax_ids).await.unwrap_or_default();
    let tax_items = taxes
        .iter()
        .map(|t| ManyToManyItem {
            key: t.id.to_string(),
            value: tax_label(t),
        })
        .collect();

    let invoice_lines_preview = invoice_line_editor_preview_json(db).await;

    DraftFormContext {
        customer_display,
        payment_term_display,
        tax_items,
        invoice_lines_preview,
    }
}

pub async fn create_get(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    let ctx_data = load_draft_form_context(&state.db, 0, 0, &[], &ctx.timezone).await;
    let form = DraftInvoiceForm {
        number: String::new(),
        reference: String::new(),
        payment_reference: String::new(),
        bank_account: String::new(),
        datetime: ctx.format_datetime_local_input(Utc::now()),
        customer_id: 0,
        payment_term_mode: PAYMENT_TERM_MODE_TERM.to_string(),
        payment_term_id: 0,
        payment_due_date: String::new(),
        taxes: vec![],
        invoice_lines_json: default_lines_json(),
    };
    let page = DraftInvoiceFormPage {
        id: 0,
        title: "Create draft invoice".to_string(),
        form,
        action_href: "/finance-invoices/create/".to_string(),
        error: None,
        can_edit: require_superuser(&ctx),
        customer_display: ctx_data.customer_display,
        payment_term_display: ctx_data.payment_term_display,
        tax_items: ctx_data.tax_items,
        invoice_lines_preview: ctx_data.invoice_lines_preview,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn create_post(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    HtmlFormBody(form): HtmlFormBody<DraftInvoiceForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/").into_response();
    }
    match form_to_input(&form, &ctx.timezone) {
        Ok(input) => match create_draft_invoice(&state.db, input).await {
            Ok(d) => Redirect::to(&format!("/finance-invoices/i/{}/", d.id)).into_response(),
            Err(e) => Redirect::to(&format!("/finance-invoices/create/?error={e}")).into_response(),
        },
        Err(e) => Redirect::to(&format!("/finance-invoices/create/?error={e}")).into_response(),
    }
}

pub async fn detail(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    Query(query): Query<DetailQuery>,
) -> Response {
    let draft = DraftInvoiceEntity::find_by_id(id)
        .filter(draft_invoice::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .ok()
        .flatten();
    let page = if let Some(d) = draft {
        let tax_ids = load_draft_invoice_tax_ids(&state.db, d.id)
            .await
            .unwrap_or_default();
        let taxes = load_taxes_by_ids(&state.db, &tax_ids)
            .await
            .unwrap_or_default();
        let tax_labels = if taxes.is_empty() {
            "—".to_string()
        } else {
            taxes.iter().map(tax_label).collect::<Vec<_>>().join(", ")
        };

        let customer_name = CustomerEntity::find_by_id(d.customer_id)
            .filter(customer::Column::DeletedAt.is_null())
            .one(&state.db)
            .await
            .ok()
            .flatten()
            .map(|c| c.name)
            .unwrap_or_else(|| format!("#{}", d.customer_id));

        let payment_term_summary = if let Ok(Some(pt)) =
            PaymentTermEntity::find_by_id(d.payment_term_id).one(&state.db).await
        {
            payment_term_summary(&state.db, &pt, &ctx.timezone).await
        } else {
            format!("#{}", d.payment_term_id)
        };

        let line_rows = draft_invoice_line_display_rows(&state.db, d.id).await;

        DraftInvoiceDetailPage {
            id: d.id,
            number: d.number.unwrap_or_else(|| "—".to_string()),
            reference: optional_display(&d.reference),
            payment_reference: optional_display(&d.payment_reference),
            bank_account: optional_display(&d.bank_account),
            datetime: ctx.format_datetime_short(d.datetime),
            customer_name,
            payment_term_summary,
            tax_labels,
            line_rows,
            can_edit: require_superuser(&ctx),
            error: query.error.filter(|e| !e.is_empty()),
        }
    } else {
        DraftInvoiceDetailPage {
            id,
            number: "Not found".to_string(),
            reference: String::new(),
            payment_reference: String::new(),
            bank_account: String::new(),
            datetime: String::new(),
            customer_name: String::new(),
            payment_term_summary: String::new(),
            tax_labels: String::new(),
            line_rows: vec![],
            can_edit: false,
            error: query.error.filter(|e| !e.is_empty()),
        }
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_get(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> maud::Markup {
    let draft = DraftInvoiceEntity::find_by_id(id)
        .filter(draft_invoice::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .ok()
        .flatten();

    let (form, ctx_data) = if let Some(d) = draft {
        let tax_ids = load_draft_invoice_tax_ids(&state.db, d.id)
            .await
            .unwrap_or_default();
        let ctx_data = load_draft_form_context(
            &state.db,
            d.customer_id,
            d.payment_term_id,
            &tax_ids,
            &ctx.timezone,
        )
        .await;
        let lines_json = draft_lines_form_json(&state.db, d.id).await;
        (
            DraftInvoiceForm {
                number: d.number.unwrap_or_default(),
                reference: d.reference.unwrap_or_default(),
                payment_reference: d.payment_reference.unwrap_or_default(),
                bank_account: d.bank_account.unwrap_or_default(),
                datetime: ctx.format_datetime_local_input(d.datetime),
                customer_id: d.customer_id,
                payment_term_mode: PAYMENT_TERM_MODE_TERM.to_string(),
                payment_term_id: d.payment_term_id,
                payment_due_date: String::new(),
                taxes: tax_ids,
                invoice_lines_json: lines_json,
            },
            ctx_data,
        )
    } else {
        let ctx_data = load_draft_form_context(&state.db, 0, 0, &[], &ctx.timezone).await;
        (
            DraftInvoiceForm {
                number: String::new(),
                reference: String::new(),
                payment_reference: String::new(),
                bank_account: String::new(),
                datetime: String::new(),
                customer_id: 0,
                payment_term_mode: PAYMENT_TERM_MODE_TERM.to_string(),
                payment_term_id: 0,
                payment_due_date: String::new(),
                taxes: vec![],
                invoice_lines_json: default_lines_json(),
            },
            ctx_data,
        )
    };

    let page = DraftInvoiceFormPage {
        id,
        title: format!("Edit draft invoice #{id}"),
        form,
        action_href: format!("/finance-invoices/i/{id}/edit/"),
        error: None,
        can_edit: require_superuser(&ctx),
        customer_display: ctx_data.customer_display,
        payment_term_display: ctx_data.payment_term_display,
        tax_items: ctx_data.tax_items,
        invoice_lines_preview: ctx_data.invoice_lines_preview,
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn edit_post(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<DraftInvoiceForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to(&format!("/finance-invoices/i/{id}/")).into_response();
    }
    match form_to_input(&form, &ctx.timezone) {
        Ok(input) => {
            let update = UpdateDraftInput {
                number: input.number,
                reference: input.reference,
                payment_reference: input.payment_reference,
                bank_account: input.bank_account,
                datetime: input.datetime,
                customer_id: input.customer_id,
                payment_term: input.payment_term,
                header_tax_ids: input.header_tax_ids,
                lines: input.lines,
            };
            match update_draft_invoice(&state.db, id, update).await {
                Ok(_) => Redirect::to(&format!("/finance-invoices/i/{id}/")).into_response(),
                Err(_) => {
                    Redirect::to(&format!("/finance-invoices/i/{id}/edit/")).into_response()
                }
            }
        }
        Err(_) => Redirect::to(&format!("/finance-invoices/i/{id}/edit/")).into_response(),
    }
}

pub async fn delete_post(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/").into_response();
    }
    let _ = soft_delete_draft(&state.db, id).await;
    Redirect::to("/finance-invoices/").into_response()
}

pub async fn post_invoice(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/").into_response();
    }
    match crate::logic::draft_new_posted(&state.db, id, Utc::now()).await {
        Ok(p) => Redirect::to(&format!("/finance-invoices/posted/{}/", p.id)).into_response(),
        Err(e) => Redirect::to(
            &DraftInvoiceDetailRouteTag::new(id)
                .with_query()
                .query("error", &e)
                .build(),
        )
        .into_response(),
    }
}
