#![feature(impl_trait_in_assoc_type)]

//! Gandola Manager plugin — gandolas, sites, and product settings.

pub mod apps;
pub mod cli;
pub mod create_modals;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod invoice_site_pos;
pub mod invoice_sites;
pub mod keys;
pub mod logic;
pub mod migrations;
pub mod import;
pub mod import_cmd;
pub mod po_from_pdf;
pub mod po_import_queue;
pub mod po_line_editor;
pub mod po_lines;
pub mod po_payment_term;
pub mod po_persist;
pub mod routes;
pub mod rune_env;
pub mod scope;
pub mod site_status;
pub mod skill_seed;
pub mod state;
pub mod templates;
pub mod tools;

use frunk::{HCons, hlist::HList};

use lariv_rs::{
    app::{App, MountedApp},
    capability::CapStore,
    db::{DbCap, DbTag},
    hooks::{AttachState, RunSeed},
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::{GetByCapTag, GetByTag},
    },
};

use state::GandolaManagerState;

pub struct GandolaManagerTag;

lariv_rs::define_passthrough_cap!(
    GandolaManagerStateCap,
    GandolaManagerTag,
    GandolaManagerState
);

lariv_rs::define_plugin_install! {
    plugin: GandolaManagerTag;
    steps: [
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        tools(tools::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        http(routes::Hook),
        state(StateHook),
        seeds(SeedsHook),
        commands(cli::Hook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, TagProof> AttachState<L, (DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<GandolaManagerTag, TagProof>,
{
    type Output = HCons<GandolaManagerStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        crate::invoice_sites::register();
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(GandolaManagerState::new(conn)))
    }
}

/// Seeds the site purchase-order invoicing skill if it is not already present.
#[derive(Clone, Copy, Default)]
pub struct SeedsHook;

#[async_trait::async_trait]
impl<M, Idx> RunSeed<M, Idx> for SeedsHook
where
    M: GetByTag<GandolaManagerTag, Idx, Value = GandolaManagerState> + Sync,
{
    async fn run_seed(app: &MountedApp<M>) -> anyhow::Result<()> {
        crate::skill_seed::ensure_all_skills(
            &app.get_capability_output::<GandolaManagerTag, Idx>().db,
        )
        .await
    }
}
