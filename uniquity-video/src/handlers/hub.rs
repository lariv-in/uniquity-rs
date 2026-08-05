use axum::response::{IntoResponse, Response};

use lariv_rs::{
    components::SharedChromeFolder,
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};

use crate::templates::HubPage;

pub async fn hub(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Response {
    let page = HubPage;
    html_built_page_or_app_layout(&page, &htmx, &chrome, &lariv_rs::components::SlotCtx::from_auth(&ctx))
        .into_response()
}
