//! CLI subcommands registered via `define_plugin_install! { commands(...) }`.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use frunk::{HCons, hlist::HList};
use lariv_rs::{
    app::MountedApp,
    command::{CommandCapability, CommandRegistrar, RunCommand},
    plugins::filesystem::{FilesystemTag, state::FilesystemState},
    tag::Tagged,
    traits::get::GetByTag,
};

use crate::{
    GandolaManagerTag,
    import_cmd::{run_import_po_pdfs, run_import_sites},
    state::GandolaManagerState,
};

pub struct GandolaImportCommandTag;

#[derive(Clone, Copy, Debug, Default)]
pub struct GandolaImportCommand;

#[derive(Subcommand, Debug, Clone)]
pub enum GandolaImportSubcommand {
    /// Import sites into the sites table.
    Sites(ImportSitesArgs),
    /// Import purchase order PDFs matched by PO number in filenames.
    PoPdfs(ImportPoPdfsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GandolaImportArgs {
    #[command(subcommand)]
    pub command: GandolaImportSubcommand,
}

#[derive(Args, Debug, Clone)]
pub struct ImportSitesArgs {
    #[arg(long, default_value = "scripts/gandola-import/sites.csv")]
    pub sites: PathBuf,
    #[arg(long, default_value = "scripts/gandola-import/customers_map.csv")]
    pub customers: PathBuf,
    #[arg(long, default_value = "scripts/gandola-import/gandolas.csv")]
    pub gandolas: PathBuf,
    #[arg(long, default_value = "scripts/gandola-import/gandola_sites.csv")]
    pub gandola_sites: PathBuf,
    /// Link every site to one gandola when `gandola_sites.csv` is missing.
    #[arg(long)]
    pub gandola_id: Option<i64>,
    #[arg(long)]
    pub dry_run: bool,
    /// Create Lariv customers that are missing from customers_map.csv lookups.
    #[arg(long, default_value_t = true)]
    pub create_missing_customers: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ImportPoPdfsArgs {
    #[arg(long)]
    pub pdf_dir: PathBuf,
    #[arg(long, default_value = "scripts/gandola-import/sites.csv")]
    pub sites: PathBuf,
    #[arg(long, default_value = "scripts/gandola-import/customers_map.csv")]
    pub customers: PathBuf,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub recursive: bool,
    #[arg(long)]
    pub dry_run: bool,
}

#[async_trait::async_trait]
impl<M, GmIdx, FsIdx> RunCommand<M, (GmIdx, FsIdx)> for GandolaImportCommand
where
    M: GetByTag<GandolaManagerTag, GmIdx, Value = GandolaManagerState> + Send + Sync + 'static,
    M: GetByTag<FilesystemTag, FsIdx, Value = FilesystemState> + Send + Sync + 'static,
    GmIdx: Send + Sync + 'static,
    FsIdx: Send + Sync + 'static,
{
    type Args = GandolaImportArgs;
    const NAME: &'static str = "gandola-import";
    const ABOUT: &'static str = "Import Gandola sites and purchase order PDFs";

    async fn run(args: Self::Args, app: MountedApp<M>) -> anyhow::Result<()> {
        let gm = app.get_capability_output::<GandolaManagerTag, GmIdx>();
        let db = gm.db.clone();

        match args.command {
            GandolaImportSubcommand::Sites(sites_args) => {
                run_import_sites(
                    &db,
                    &sites_args.sites,
                    &sites_args.customers,
                    &sites_args.gandolas,
                    &sites_args.gandola_sites,
                    sites_args.gandola_id,
                    sites_args.dry_run,
                    sites_args.create_missing_customers,
                )
                .await?;
            }
            GandolaImportSubcommand::PoPdfs(po_args) => {
                let fs = app.get_capability_output::<FilesystemTag, FsIdx>().clone();
                let out = po_args
                    .out
                    .unwrap_or_else(|| PathBuf::from("po_import_report.json"));
                run_import_po_pdfs(
                    &db,
                    &fs,
                    &po_args.sites,
                    &po_args.customers,
                    &po_args.pdf_dir,
                    &out,
                    po_args.recursive,
                    po_args.dry_run,
                )
                .await?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
pub struct Hook;

impl<C> CommandRegistrar<C> for Hook
where
    C: HList + Clone,
{
    type Output = GandolaManagerCommands<C>;

    fn register_commands(self, cap: CommandCapability<C>) -> CommandCapability<Self::Output> {
        cap.prepend::<GandolaImportCommandTag, _>(GandolaImportCommand)
    }
}

pub type GandolaManagerCommands<C> =
    HCons<Tagged<GandolaImportCommandTag, GandolaImportCommand>, C>;

