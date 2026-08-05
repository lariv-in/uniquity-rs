use std::collections::HashMap;

use axum::{Form, response::{IntoResponse, Redirect, Response}};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    html_form::FormFieldKey,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};

use uniquity_common::require_superuser;

use crate::{
    accounting_preferences_patch::save_accounting_preferences_addons,
    entities::accounting_preferences::{self, Entity as AccountingPreferencesEntity},
    forms::AccountingPreferencesFormField,
    logic::journal::load_accounting_preferences,
    routes::AccountingPreferencesRouteTag,
    state::AccountsState,
    templates::AccountingPreferencesPage,
};

use super::util::opt_str;

pub async fn get(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance").into_response();
    }
    let prefs = load_accounting_preferences(&state.db).await;
    let addon_inputs =
        crate::accounting_preferences_patch::render_accounting_preferences_addons(&state.db).await;
    let page = AccountingPreferencesPage {
        invoice_number_format: prefs.invoice_number_format.unwrap_or_default(),
        invoice_pdf_template: prefs.invoice_pdf_template.unwrap_or_default(),
        addon_inputs,
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn post(
    Cap(state): Cap<AccountsState>,
    RequireAuth(ctx): RequireAuth,
    Form(params): Form<HashMap<String, String>>,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance").into_response();
    }
    let now = Utc::now();
    let existing = load_accounting_preferences(&state.db).await;
    let invoice_number_format = params
        .get(AccountingPreferencesFormField::InvoiceNumberFormat.html_name())
        .map(|s| s.as_str())
        .unwrap_or("");
    let invoice_pdf_template = params
        .get(AccountingPreferencesFormField::InvoicePdfTemplate.html_name())
        .map(|s| s.as_str())
        .unwrap_or("");
    let model = accounting_preferences::ActiveModel {
        id: Set(existing.id.max(1)),
        updated_at: Set(Some(now)),
        invoice_number_format: Set(opt_str(invoice_number_format)),
        invoice_pdf_template: Set(opt_str(invoice_pdf_template)),
        ..Default::default()
    };
    if AccountingPreferencesEntity::find_by_id(existing.id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        if let Err(e) = model.update(&state.db).await {
            tracing::error!("accounting preferences update: {e}");
            return Redirect::to(&AccountingPreferencesRouteTag.url()).into_response();
        }
    } else {
        let _ = accounting_preferences::ActiveModel {
            id: Set(1),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            invoice_number_format: Set(opt_str(invoice_number_format)),
            invoice_pdf_template: Set(opt_str(invoice_pdf_template)),
            ..Default::default()
        }
        .insert(&state.db)
        .await;
    }
    if let Err(e) = save_accounting_preferences_addons(&state.db, &params).await {
        tracing::error!("accounting preferences addon save: {e}");
    }
    Redirect::to(&AccountingPreferencesRouteTag.url()).into_response()
}
