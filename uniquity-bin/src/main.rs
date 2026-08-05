#![recursion_limit = "512"]

use lariv_rs::app::App;
use lariv_rs::plugins::{dashboard, filesystem, llm_assistant, otp, pwa, users};
use tracing_subscriber::EnvFilter;

/// Deep HList install/mount chains for this deployment overflow the default ~8MB stack.
const STACK_SIZE: usize = 16 * 1024 * 1024;

async fn run() -> anyhow::Result<()> {
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

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run().await?;
    Ok(())
}

fn main() {
    let result = std::thread::Builder::new()
        .name("uniquity-server".into())
        .stack_size(STACK_SIZE)
        .spawn(|| {
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::from_default_env().add_directive("warn".parse().expect("directive")),
                )
                .init();

            // current_thread keeps install/mount on this large-stack thread; a
            // multi_thread runtime would resume after `.await` on ~2MB worker stacks.
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime")
                .block_on(run())
        })
        .expect("spawn server thread")
        .join();

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("server thread panicked");
            std::process::exit(1);
        }
    }
}
