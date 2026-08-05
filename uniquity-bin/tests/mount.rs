//! Compile smoke test for the Uniquity deployment plugin stack.

#![recursion_limit = "512"]

use std::path::PathBuf;

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, llm_assistant, otp, pwa, users};

const MINIMAL_DB_TOML: &str = r#"database_url = "sqlite::memory:"
[users]
adminEmail = "admin@test.local"
adminPassword = "adminadmin"
signingKey = "dGVzdC1zaWduaW5nLWtleS1wYWRkZWQtdG8tNjQtYnl0ZXMhISEhISEhISEhISE="
jwtIssuer = "dW5pcXVpdHktdGVzdC1pc3N1ZXItcGFkZGVkLXRvLTY0LWJ5dGVzIQ=="
"#;

fn temp_config(body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "uniquity-mount-{}-{}.toml",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::write(&path, body).expect("write temp config");
    path
}

#[tokio::test]
async fn uniquity_stack_mounts() {
    let handle = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime")
                .block_on(async {
                    let app = App::new_web_app();
                    let app = users::install(app);
                    let app = filesystem::install(app);
                    let app = llm_assistant::install(app);
                    let app = uniquity_finance_accounts::install(app);
                    let app = uniquity_finance_customer::install(app);
                    let app = uniquity_finance_creditnotes::install(app);
                    let app = uniquity_finance_fiscal_year::install(app);
                    let app = uniquity_finance_taxes::install(app);
                    let app = uniquity_finance_products::install(app);
                    let app = uniquity_finance_invoices::install(app);
                    let app = uniquity_finance_indian::install(app);
                    let app = uniquity_employees::install(app);
                    let app = uniquity_video::install(app);
                    let app = otp::install(app);
                    let app = pwa::install(app);
                    let app = dashboard::install(app);

                    let path = temp_config(MINIMAL_DB_TOML);
                    let app = app.load_config(&path).await.expect("load_config");
                    std::fs::remove_file(&path).ok();
                    let _mounted = app.mount();
                });
        })
        .expect("spawn mount thread");
    handle.join().expect("join mount thread");
}
