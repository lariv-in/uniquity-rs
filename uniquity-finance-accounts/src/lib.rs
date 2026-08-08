#![feature(impl_trait_in_assoc_type)]

//! Uniquity finance accounts plugin.

pub mod account_select;
pub mod account_validation;
pub mod accounting_detail_menu;
pub mod accounting_preferences_patch;
pub mod accounting_sidebar;
pub mod apps;
pub mod balance_type;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod journal_type;
pub mod keys;
pub mod logic;
pub mod migrations;
pub mod routes;
pub mod scope;
pub mod source_doc_label;
pub mod source_doc_registry;
pub mod state;
pub mod templates;

pub use account_select::account_select_url_with_balance_type as account_select_route_url;
pub use account_validation::validate_leaf_account_balance_type;
pub use balance_type::BalanceType;
pub use journal_type::JournalType;
pub use source_doc_label::{source_doc_ref_summary, source_doc_summary, source_doc_type_label};
pub use source_doc_registry::{SourceDocInstance, SourceDocRegistry, SourceDocType};
pub use state::AccountsState;

pub use crate::apps::ACCOUNTING_APP_KEY;

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

pub struct UniquityFinanceAccountsTag;

lariv_rs::define_passthrough_cap!(
    UniquityFinanceAccountsStateCap,
    UniquityFinanceAccountsTag,
    AccountsState
);

lariv_rs::define_plugin_install! {
    plugin: UniquityFinanceAccountsTag;
    steps: [
        cap_attach(accounting_sidebar::AccountingSidebarTag, accounting_sidebar::AccountingSidebarCap, accounting_sidebar::AccountingSidebarCap::<frunk::HNil>::new()),
        cap_hook(accounting_sidebar::AccountingSidebarTag, accounting_sidebar::AccountingSidebarCap, accounting_sidebar::BaseHook),
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
    L: HList + CapTagAbsent<UniquityFinanceAccountsTag, TagProof>,
{
    type Output = HCons<UniquityFinanceAccountsStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(AccountsState::new(conn)))
    }
}
