#![feature(impl_trait_in_assoc_type)]

//! Uniquity employees plugin.

pub mod apps;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod migrations;
pub mod routes;
pub mod scope;
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

use state::EmployeesState;

pub struct UniquityEmployeesTag;

lariv_rs::define_passthrough_cap!(
    UniquityEmployeesStateCap,
    UniquityEmployeesTag,
    EmployeesState
);

lariv_rs::define_plugin_install! {
    plugin: UniquityEmployeesTag;
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
    L: HList + CapTagAbsent<UniquityEmployeesTag, TagProof>,
{
    type Output = HCons<UniquityEmployeesStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(EmployeesState::new(conn)))
    }
}
