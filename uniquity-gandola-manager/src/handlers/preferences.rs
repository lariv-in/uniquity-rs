use axum::{
    Form,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::finance_invoices::logic::default_payment_term_lines_json,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout, html_built_page_with_slots},
};

use crate::{
    entities::preferences,
    forms::GandolaPreferencesForm,
    scope::{is_superuser, load_preferences, parse_optional_i64, product_name},
    state::GandolaManagerState,
    templates::GandolaPreferencesPage,
};

const LIST_URL: &str = "/gandola/";

fn product_id_str(id: Option<i64>) -> String {
    id.filter(|&id| id > 0)
        .map(|id| id.to_string())
        .unwrap_or_default()
}

async fn page_from_prefs(
    db: &sea_orm::DatabaseConnection,
    prefs: &preferences::Model,
    error: String,
    can_edit: bool,
) -> GandolaPreferencesPage {
    GandolaPreferencesPage {
        gandola_product_id: product_id_str(prefs.gandola_product_id),
        gandola_product_display: product_name(db, prefs.gandola_product_id).await,
        tpi_product_id: product_id_str(prefs.tpi_product_id),
        tpi_product_display: product_name(db, prefs.tpi_product_id).await,
        dti_product_id: product_id_str(prefs.dti_product_id),
        dti_product_display: product_name(db, prefs.dti_product_id).await,
        payment_term_lines_json: prefs
            .payment_term_lines_json
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(default_payment_term_lines_json),
        error,
        can_edit,
    }
}

pub async fn get(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    let prefs = load_preferences(&state.db).await;
    let page = page_from_prefs(&state.db, &prefs, String::new(), is_superuser(&ctx)).await;
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn post(
    Cap(state): Cap<GandolaManagerState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Form(form): Form<GandolaPreferencesForm>,
) -> Response {
    if !is_superuser(&ctx) {
        return Redirect::to(LIST_URL).into_response();
    }
    let existing = load_preferences(&state.db).await;
    let now = Utc::now();
    let payment_term = if form.payment_term_lines_json.trim().is_empty() {
        default_payment_term_lines_json()
    } else {
        form.payment_term_lines_json.clone()
    };
    let model = preferences::ActiveModel {
        id: Set(existing.id),
        gandola_product_id: Set(parse_optional_i64(&form.gandola_product_id)),
        tpi_product_id: Set(parse_optional_i64(&form.tpi_product_id)),
        dti_product_id: Set(parse_optional_i64(&form.dti_product_id)),
        payment_term_lines_json: Set(Some(payment_term)),
        updated_at: Set(Some(now)),
        ..Default::default()
    };
    match model.update(&state.db).await {
        Ok(saved) => {
            let page = page_from_prefs(&state.db, &saved, String::new(), true).await;
            html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx))
                .into_response()
        }
        Err(_) => {
            let insert = preferences::ActiveModel {
                id: Set(1),
                created_at: Set(Some(now)),
                updated_at: Set(Some(now)),
                gandola_product_id: Set(parse_optional_i64(&form.gandola_product_id)),
                tpi_product_id: Set(parse_optional_i64(&form.tpi_product_id)),
                dti_product_id: Set(parse_optional_i64(&form.dti_product_id)),
                payment_term_lines_json: Set(Some(
                    if form.payment_term_lines_json.trim().is_empty() {
                        default_payment_term_lines_json()
                    } else {
                        form.payment_term_lines_json.clone()
                    },
                )),
            };
            match insert.insert(&state.db).await {
                Ok(saved) => {
                    let page = page_from_prefs(&state.db, &saved, String::new(), true).await;
                    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx))
                        .into_response()
                }
                Err(e) => {
                    let prefs = preferences::Model {
                        id: 1,
                        created_at: Some(now),
                        updated_at: Some(now),
                        gandola_product_id: parse_optional_i64(&form.gandola_product_id),
                        tpi_product_id: parse_optional_i64(&form.tpi_product_id),
                        dti_product_id: parse_optional_i64(&form.dti_product_id),
                        payment_term_lines_json: Some(form.payment_term_lines_json.clone()),
                    };
                    let page = page_from_prefs(&state.db, &prefs, e.to_string(), true).await;
                    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
                        .into_response()
                }
            }
        }
    }
}
