#![feature(impl_trait_in_assoc_type)]

//! Gandola Manager plugin — gandolas, sites, and product settings.

pub mod apps;
pub mod create_modals;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod invoice_sites;
pub mod keys;
pub mod logic;
pub mod migrations;
pub mod routes;
pub mod scope;
pub mod site_status;
pub mod state;
pub mod templates;

use frunk::{HCons, hlist::HList};

use lariv_rs::{
    app::App,
    capability::CapStore,
    db::{DbCap, DbTag},
    hooks::AttachState,
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::GetByCapTag,
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
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        http(routes::Hook),
        state(StateHook),
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
