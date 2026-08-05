use axum::{
    extract::Path,
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};

use uniquity_common::require_superuser;

use crate::{
    entities::{
        paid_invoice::{self, Entity as PaidInvoiceEntity},
        partially_paid_invoice::{self, Entity as PartiallyPaidInvoiceEntity},
        payment::{self, Entity as PaymentEntity},
        payment_term::Entity as PaymentTermEntity,
        posted_invoice::{self, Entity as PostedInvoiceEntity},
    },
    logic::{
        invoice_line_editor::{
            invoice_customer_name, invoice_header_tax_labels, posted_invoice_line_display_rows,
        },
        optional_display,
        payment_term::payment_term_summary,
        posted_invoice_can_accept_payment,
        tax_assoc::load_posted_invoice_tax_ids,
    },
    state::InvoicesState,
    templates::{PaidInvoiceDetailPage, PartiallyPaidInvoiceDetailPage, SettlementDetailContext},
};

async fn load_settlement_context(
    db: &sea_orm::DatabaseConnection,
    settlement_id: i64,
    payment_id: i64,
    posted_invoice_id: i64,
    prior_partially_paid_invoice_id: Option<i64>,
    tz: &str,
) -> Option<SettlementDetailContext> {
    let posted = PostedInvoiceEntity::find_by_id(posted_invoice_id)
        .filter(posted_invoice::Column::DeletedAt.is_null())
        .one(db)
        .await
        .ok()
        .flatten()?;
    let payment = PaymentEntity::find_by_id(payment_id)
        .filter(payment::Column::DeletedAt.is_null())
        .one(db)
        .await
        .ok()
        .flatten()?;
    let tax_ids = load_posted_invoice_tax_ids(db, posted.id)
        .await
        .unwrap_or_default();
    let tax_labels = invoice_header_tax_labels(db, &tax_ids).await;
    let customer_name = invoice_customer_name(db, posted.customer_id).await;
    let payment_term_summary = if let Ok(Some(pt)) =
        PaymentTermEntity::find_by_id(posted.payment_term_id).one(db).await
    {
        payment_term_summary(db, &pt, tz).await
    } else {
        format!("#{}", posted.payment_term_id)
    };
    let line_rows = posted_invoice_line_display_rows(db, posted.id).await;
    let payment_amount = uniquity_common::decimal::decimal_display(payment.amount);
    let payment_label = format!("#{} · {payment_amount}", payment.id);
    let payment_href = format!("/finance-invoices/payments/{}/", payment.id);
    let prior_partial_label = prior_partially_paid_invoice_id
        .filter(|id| *id > 0)
        .map(|id| format!("#{id}"));
    let prior_partial_href = prior_partially_paid_invoice_id
        .filter(|id| *id > 0)
        .map(|id| format!("/finance-invoices/partial/{id}/"));
    Some(SettlementDetailContext {
        settlement_id,
        posted_invoice_id: posted.id,
        number: posted.number,
        reference: optional_display(&posted.reference),
        payment_reference: optional_display(&posted.payment_reference),
        bank_account: optional_display(&posted.bank_account),
        datetime: lariv_rs::datetime::format_datetime_short(posted.datetime, tz),
        posted_at: posted
            .posted_at
            .map(|t| lariv_rs::datetime::format_datetime_short(t, tz)),
        customer_name,
        payment_term_summary,
        tax_labels,
        line_rows,
        journal_entry_id: posted.journal_entry_id,
        payment_id: payment.id,
        payment_label,
        payment_href,
        payment_datetime: lariv_rs::datetime::format_datetime_short(payment.datetime, tz),
        prior_partial_label,
        prior_partial_href,
    })
}

pub async fn paid_detail(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let paid = PaidInvoiceEntity::find_by_id(id)
        .filter(paid_invoice::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .ok()
        .flatten();
    let can_edit = require_superuser(&ctx);
    let page = if let Some(paid) = paid {
        if let Some(ctx_data) = load_settlement_context(
            &state.db,
            paid.id,
            paid.payment_id,
            paid.posted_invoice_id,
            paid.prior_partially_paid_invoice_id,
            &ctx.timezone,
        )
        .await
        {
            PaidInvoiceDetailPage {
                ctx: ctx_data,
                can_edit,
                can_pay: false,
            }
        } else {
            PaidInvoiceDetailPage::not_found(id)
        }
    } else {
        PaidInvoiceDetailPage::not_found(id)
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn partial_detail(
    Cap(state): Cap<InvoicesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let partial = PartiallyPaidInvoiceEntity::find_by_id(id)
        .filter(partially_paid_invoice::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .ok()
        .flatten();
    let can_edit = require_superuser(&ctx);
    let page = if let Some(partial) = partial {
        if let Some(ctx_data) = load_settlement_context(
            &state.db,
            partial.id,
            partial.payment_id,
            partial.posted_invoice_id,
            partial.prior_partially_paid_invoice_id,
            &ctx.timezone,
        )
        .await
        {
            let can_pay = can_edit
                && posted_invoice_can_accept_payment(&state.db, partial.posted_invoice_id).await;
            PartiallyPaidInvoiceDetailPage {
                ctx: ctx_data,
                can_edit,
                can_pay,
            }
        } else {
            PartiallyPaidInvoiceDetailPage::not_found(id)
        }
    } else {
        PartiallyPaidInvoiceDetailPage::not_found(id)
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}
