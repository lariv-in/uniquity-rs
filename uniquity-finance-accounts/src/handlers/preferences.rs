use std::collections::HashMap;

use axum::{
    Form,
    response::{IntoResponse, Redirect, Response},
};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};

use uniquity_common::require_superuser;

use crate::{
    accounting_preferences_patch::save_accounting_preferences_addons,
    routes::AccountingPreferencesRouteTag,
    state::AccountsState,
    templates::AccountingPreferencesPage,
};

pub async fn get(
    Cap(state): Cap<AccountsState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    if !require_superuser(&ctx) {
        return Redirect::to("/finance").into_response();
    }
    let addon_inputs =
        crate::accounting_preferences_patch::render_accounting_preferences_addons(&state.db).await;
    let page = AccountingPreferencesPage { addon_inputs };
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
    if let Err(e) = save_accounting_preferences_addons(&state.db, &params).await {
        tracing::error!("accounting preferences addon save: {e}");
    }
    Redirect::to(&AccountingPreferencesRouteTag.url()).into_response()
}
