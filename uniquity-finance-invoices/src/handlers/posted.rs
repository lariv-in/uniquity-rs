use axum::{
    Form,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout, html_built_page_with_slots},
};

use uniquity_common::require_superuser;

use crate::{
    entities::{
        payment_term::Entity as PaymentTermEntity,
        posted_invoice::{self, Entity as PostedInvoiceEntity},
    },
    forms::CancelInvoiceForm,
    logic::{
        invoice_line_editor::{
            invoice_customer_name, invoice_header_tax_labels, posted_invoice_line_display_rows,
        },
        optional_display,
        payment_term::payment_term_summary,
        posted_invoice_can_accept_payment,
        posted_new_cancelled,
        tax_assoc::load_posted_invoice_tax_ids,
    },
    state::InvoicesState,
    templates::{CancelInvoicePage, PostedInvoiceDetailPage},
};

pub async fn detail(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let posted = PostedInvoiceEntity::find_by_id(id)
        .filter(posted_invoice::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .ok()
        .flatten();
    let page = if let Some(p) = posted {
        let tax_ids = load_posted_invoice_tax_ids(&state.db, p.id)
            .await
            .unwrap_or_default();
        let tax_labels = invoice_header_tax_labels(&state.db, &tax_ids).await;
        let customer_name = invoice_customer_name(&state.db, p.customer_id).await;
        let payment_term_summary = if let Ok(Some(pt)) =
            PaymentTermEntity::find_by_id(p.payment_term_id).one(&state.db).await
        {
            payment_term_summary(&state.db, &pt, &ctx.timezone).await
        } else {
            format!("#{}", p.payment_term_id)
        };
        let line_rows = posted_invoice_line_display_rows(&state.db, p.id).await;
        let can_edit = require_superuser(&ctx);
        let can_pay = can_edit && posted_invoice_can_accept_payment(&state.db, p.id).await;
        PostedInvoiceDetailPage {
            id: p.id,
            number: p.number,
            reference: optional_display(&p.reference),
            payment_reference: optional_display(&p.payment_reference),
            bank_account: optional_display(&p.bank_account),
            datetime: ctx.format_datetime_short(p.datetime),
            customer_name,
            payment_term_summary,
            tax_labels,
            line_rows,
            journal_entry_id: p.journal_entry_id,
            can_edit,
            can_pay,
        }
    } else {
        PostedInvoiceDetailPage {
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
            journal_entry_id: 0,
            can_edit: false,
            can_pay: false,
        }
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn cancel_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> maud::Markup {
    let page = CancelInvoicePage {
        id,
        form: CancelInvoiceForm {
            reason: String::new(),
        },
        can_edit: require_superuser(&ctx),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn cancel_invoice(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
    Form(form): Form<CancelInvoiceForm>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance-invoices/posted/").into_response();
    }
    match posted_new_cancelled(&state.db, id, form.reason, Utc::now()).await {
        Ok(c) => Redirect::to(&format!("/finance-invoices/cancelled/{}/", c.id)).into_response(),
        Err(_) => Redirect::to(&format!("/finance-invoices/posted/{id}/")).into_response(),
    }
}
