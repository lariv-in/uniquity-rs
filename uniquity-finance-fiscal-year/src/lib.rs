#![feature(impl_trait_in_assoc_type)]

//! Uniquity finance fiscal year plugin.

pub mod accounting_sidebar;
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

use state::FiscalYearState;

pub struct UniquityFinanceFiscalYearTag;

lariv_rs::define_passthrough_cap!(
    UniquityFinanceFiscalYearStateCap,
    UniquityFinanceFiscalYearTag,
    FiscalYearState
);

lariv_rs::define_plugin_install! {
    plugin: UniquityFinanceFiscalYearTag;
    steps: [
        cap_hook(uniquity_finance_accounts::accounting_sidebar::AccountingSidebarTag, uniquity_finance_accounts::accounting_sidebar::AccountingSidebarCap, accounting_sidebar::Hook),
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
    L: HList + CapTagAbsent<UniquityFinanceFiscalYearTag, TagProof>,
{
    type Output = HCons<UniquityFinanceFiscalYearStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(FiscalYearState::new(conn)))
    }
}
