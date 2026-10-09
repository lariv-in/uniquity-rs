#![recursion_limit = "4096"]

use lariv_rs::app::App;
use lariv_rs::plugins::{
    contacts, crm, customer, dashboard, filesystem, finance_accounts, finance_creditnotes,
    finance_customer, finance_indian, finance_invoices, finance_products, finance_taxes,
    llm_assistant, otp, pwa, tasks, users, website,
};
use tracing_subscriber::EnvFilter;

mod hr_role;
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
    // Before CRM so contacts tables exist for lead foreign keys.
    let app = contacts::install(app);
    // Before CRM so `tasks` can copy `crm_tasks` before CRM drops those tables.
    let app = tasks::install(app);
    let app = crm::install(app);
    let app = finance_customer::install(app);
    let app = finance_creditnotes::install(app);
    let app = finance_taxes::install(app);
    let app = finance_products::install(app);
    let app = finance_invoices::install(app);
    let app = finance_indian::install(app);
    let app = uniquity_gandola_manager::install(app);
    // After finance and gandola so `hr` is appended to allowlists those plugins already registered.
    let app = hr_role::install(app);
    let app = otp::install(app);
    let app = dashboard::install(app);
    // After dashboard so website can own `/` (CMS home) over the auth redirect.
    let app = website::install(app);
    let app = pwa::install(app);
    let app = website_seed::install(app);

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run_migrations().await?;
    app.run().await?;
    Ok(())
}
