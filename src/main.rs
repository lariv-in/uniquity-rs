#![recursion_limit = "512"]

use lariv_rs::app::App;
use lariv_rs::plugins::{
    crm, customer, dashboard, filesystem, finance_accounts, finance_creditnotes, finance_customer,
    finance_fiscal_year, finance_indian, finance_invoices, finance_products, finance_taxes,
    llm_assistant, otp, pwa, users,
};
use tracing_subscriber::EnvFilter;

/// Deep HList install/mount chains for this deployment overflow the default ~8 MiB stack.
const STACK_SIZE: usize = 64 * 1024 * 1024;

/// Raise the process stack soft limit so `thread::Builder::stack_size` is not clipped by
/// `ulimit -s` (often 8192 KiB). Without this, requesting 32–64 MiB still leaves ~8 MiB.
#[cfg(unix)]
fn raise_process_stack_limit(bytes: usize) {
    use std::mem::MaybeUninit;

    unsafe {
        let mut lim = MaybeUninit::<libc::rlimit>::uninit();
        if libc::getrlimit(libc::RLIMIT_STACK, lim.as_mut_ptr()) != 0 {
            return;
        }
        let mut lim = lim.assume_init();
        let want = bytes as libc::rlim_t;
        lim.rlim_cur = if lim.rlim_max == libc::RLIM_INFINITY {
            want
        } else {
            want.min(lim.rlim_max)
        };
        if libc::setrlimit(libc::RLIMIT_STACK, &lim) != 0 {
            eprintln!(
                "warning: could not raise stack limit to {} MiB; \
                 try `ulimit -s unlimited` before starting the server",
                bytes / (1024 * 1024)
            );
        }
    }
}

#[cfg(not(unix))]
fn raise_process_stack_limit(_bytes: usize) {}

async fn run() -> anyhow::Result<()> {
    let app = App::new_web_app();
    let app = users::install(app);
    let app = filesystem::install(app);
    let app = llm_assistant::install(app);
    let app = finance_accounts::install(app);
    let app = customer::install(app);
    let app = crm::install(app);
    let app = finance_customer::install(app);
    let app = finance_creditnotes::install(app);
    let app = finance_fiscal_year::install(app);
    let app = finance_taxes::install(app);
    let app = finance_products::install(app);
    let app = finance_invoices::install(app);
    let app = finance_indian::install(app);
    let app = uniquity_gandola_manager::install(app);
    let app = otp::install(app);
    let app = pwa::install(app);
    let app = dashboard::install(app);

    let app = app.load_config("config.toml").await?;
    let app = app.mount();
    app.run().await?;
    Ok(())
}

fn main() {
    raise_process_stack_limit(STACK_SIZE);

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
