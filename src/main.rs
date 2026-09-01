#![recursion_limit = "512"]

use lariv_rs::app::App;
use lariv_rs::plugins::{
    crm, customer, dashboard, filesystem, finance_accounts, finance_creditnotes, finance_customer,
    finance_indian, finance_invoices, finance_products, finance_taxes, llm_assistant, otp, pwa,
    users, website,
};
use tracing_subscriber::EnvFilter;

mod website_seed;

#[lariv_rs::main(
    stack_size = 64 * 1024 * 1024,
    flavor = "multi_thread",
    thread_name = "uniquity-server"
)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("warn".parse().expect("directive"))
                .add_directive("llm_assistant::imap=info".parse().expect("directive")),
        )
        .init();

    let app = App::new_web_app();
    let app = users::install(app);
    let app = filesystem::install(app);
    let app = llm_assistant::install(app);
    let app = finance_accounts::install(app);
    let app = customer::install(app);
    let app = crm::install(app);
    let app = finance_customer::install(app);
    let app = finance_creditnotes::install(app);
    let app = finance_taxes::install(app);
    let app = finance_products::install(app);
    let app = finance_invoices::install(app);
    let app = finance_indian::install(app);
    let app = uniquity_gandola_manager::install(app);
    let app = otp::install(app);
    let app = pwa::install(app);
    let app = dashboard::install(app);
    // After dashboard so website can own `/` (CMS home) over the auth redirect.
    let app = website::install(app);

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run_migrations().await?;
    app.run_seeds().await?;
    let website = app.get_capability_output::<website::WebsiteTag, _>();
    tracing::info!("uniquity website: seeding homepage and media");
    website_seed::ensure_homepage(website).await?;
    tracing::info!("uniquity website: seed complete");
    app.run().await?;
    Ok(())
}
