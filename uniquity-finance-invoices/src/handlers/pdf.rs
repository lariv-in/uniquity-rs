use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use sea_orm::EntityTrait;

use lariv_rs::{
    http::Cap,
    plugins::users::middleware::RequireAuth,
};

use uniquity_common::require_superuser;

use crate::logic::invoice_pdf::{
    InvoicePdfError, render_cancelled_invoice_pdf, render_draft_invoice_pdf,
    render_paid_invoice_pdf, render_partially_paid_invoice_pdf, render_posted_invoice_pdf,
};
use crate::{
    entities::PostedInvoiceEntity,
    state::InvoicesState,
};

fn pdf_error_response(err: InvoicePdfError) -> Response {
    match err {
        InvoicePdfError::NotFound => (StatusCode::NOT_FOUND, "Invoice not found").into_response(),
        InvoicePdfError::Message(msg) if msg.contains("Configure the invoice PDF template") => {
            (StatusCode::BAD_REQUEST, msg).into_response()
        }
        InvoicePdfError::Message(msg) => {
            tracing::error!("invoice pdf: {msg}");
            (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
        }
    }
}

fn pdf_ok_response(result: crate::logic::invoice_pdf::InvoicePdfResult) -> Response {
    let filename = format!("{}.pdf", result.filename_base);
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(result.bytes),
    )
        .into_response()
}

pub async fn draft_pdf(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return StatusCode::FORBIDDEN.into_response();
    }
    match render_draft_invoice_pdf(&state.db, id, &ctx.timezone).await {
        Ok(result) => pdf_ok_response(result),
        Err(e) => pdf_error_response(e),
    }
}

pub async fn posted_pdf(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let posted = match PostedInvoiceEntity::find_by_id(id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    {
        Some(p) => p,
        None => return pdf_error_response(InvoicePdfError::NotFound),
    };
    match render_posted_invoice_pdf(&state.db, posted, &ctx.timezone).await {
        Ok(result) => pdf_ok_response(result),
        Err(e) => pdf_error_response(e),
    }
}

pub async fn cancelled_pdf(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return StatusCode::FORBIDDEN.into_response();
    }
    match render_cancelled_invoice_pdf(&state.db, id, &ctx.timezone).await {
        Ok(result) => pdf_ok_response(result),
        Err(e) => pdf_error_response(e),
    }
}

pub async fn paid_pdf(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return StatusCode::FORBIDDEN.into_response();
    }
    match render_paid_invoice_pdf(&state.db, id, &ctx.timezone).await {
        Ok(result) => pdf_ok_response(result),
        Err(e) => pdf_error_response(e),
    }
}

pub async fn partially_paid_pdf(
    Cap(state): Cap<InvoicesState>,
    RequireAuth(ctx): RequireAuth,
    Path(id): Path<i64>,
) -> Response {
    if !require_superuser(&ctx) {
        return StatusCode::FORBIDDEN.into_response();
    }
    match render_partially_paid_invoice_pdf(&state.db, id, &ctx.timezone).await {
        Ok(result) => pdf_ok_response(result),
        Err(e) => pdf_error_response(e),
    }
}
