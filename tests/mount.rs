//! Compile smoke test for the Uniquity deployment plugin stack.

#![recursion_limit = "512"]

use std::path::PathBuf;

use lariv_rs::app::{App, MountedApp};
use lariv_rs::command::{BuildCli, CommandCapability, CommandTag};
use lariv_rs::traits::get::GetByTag;
use lariv_rs::plugins::{
    crm, customer, dashboard, filesystem, finance_accounts, finance_creditnotes, finance_customer,
    finance_indian, finance_invoices, finance_products, finance_taxes,
    llm_assistant, otp, pwa, users, website,
};

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

fn assert_gandola_import_cli<M, CmdIdx, Cmds, Proof>(app: MountedApp<M>)
where
    M: GetByTag<CommandTag, CmdIdx, Value = CommandCapability<Cmds>> + Send + 'static,
    Cmds: BuildCli<M, Proof>,
    CmdIdx: Send + Sync + 'static,
{
    let cmd_cap = app.get_capability_output::<CommandTag, CmdIdx>();
    let cli = cmd_cap.build_cli::<M, Proof>();
    let names: Vec<String> = cli
        .get_subcommands()
        .map(|sub| sub.get_name().to_string())
        .collect();
    assert!(
        names.contains(&"gandola-import".to_string()),
        "expected gandola-import subcommand, got: {names:?}"
    );
}

#[tokio::test]
async fn uniquity_stack_mounts() {
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
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
                    let app = website::install(app);

                    let path = temp_config(MINIMAL_DB_TOML);
                    let app = app.load_config(&path).await.expect("load_config");
                    std::fs::remove_file(&path).ok();
                    let mounted = app.mount();
                    assert_gandola_import_cli(mounted);
                });
        })
        .expect("spawn mount thread");
    handle.join().expect("join mount thread");
}
