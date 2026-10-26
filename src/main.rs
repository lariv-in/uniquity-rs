#![feature(impl_trait_in_assoc_type)]
#![recursion_limit = "4096"]

use lariv_rs::app::App;
use lariv_rs::plugins::{
    contacts, crm, customer, dashboard, filesystem, finance_accounts, finance_creditnotes,
    finance_indian, finance_invoices, finance_products, finance_purchases, finance_services,
    finance_taxes, llm_assistant, otp, pwa, users, website,
};
use tracing_subscriber::EnvFilter;

mod accountant_role;
mod invoice_line_quantity;
mod website_seed;

#[lariv_rs::main(
    stack_size = 64 * 1024 * 1024,
    flavor = "multi_thread",
    thread_name = "uniquity-server"
)]
async fn main() -> anyhow::Result<()> {
    // rustls is built with both aws-lc-rs and ring, so it will not choose a
    // process-wide provider. The rest of the stack uses aws-lc-rs.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

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
    // Tasks hooks the PWA service worker, so PWA is installed first.
    let app = pwa::install(app);
    // Before invoices. Invoice migrations copy bill-to parties into `crm_customers`.
    let app = crm::install(app);
    let app = finance_creditnotes::install(app);
    let app = finance_taxes::install(app);
    let app = finance_products::install(app);
    // Before invoices so service lines can point at the services catalog.
    let app = finance_services::install(app);
    let app = finance_invoices::install(app);
    let app = invoice_line_quantity::install(app);
    // After products and services. Purchase migrations copy rows from those catalogs.
    let app = finance_purchases::install(app);
    let app = finance_indian::install(app);
    let app = uniquity_gandola_manager::install(app);
    // After finance, customers, contacts, CRM, and gandola so `accountant` is appended to
    // allowlists those plugins already registered.
    let app = accountant_role::install(app);
    let app = otp::install(app);
    let app = dashboard::install(app);
    // After dashboard so website can own `/` (CMS home) over the auth redirect.
    let app = website::install(app);
    let app = website_seed::install(app);

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run_migrations().await?;
    app.run().await?;
    Ok(())
}
