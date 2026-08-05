//! Shared accounting app sidebar — base links plus addon plugin patches at install time.
//!
//! The hub plugin registers this capability via `cap_attach` + `cap_hook(BaseHook)` in
//! [`define_plugin_install!`](lariv_rs::plugin_install::define_plugin_install). Finance addon
//! plugins add links with `cap_hook(AccountingSidebarTag, AccountingSidebarCap, Hook)`.

use std::marker::PhantomData;
use std::sync::OnceLock;

use frunk::{HCons, HNil, hlist::HList};
use lariv_rs::{
    app::App,
    capability::{CapHookExt, Capability, HasCapTag},
    components::{
        SidebarMenu, SidebarMenuBack, SidebarMenuItem, sidebar_menu, sidebar_menu_item,
    },
    http::RouteUrl,
    plugins::dashboard::routes::DashboardAppsRouteTag,
    tag::Tagged,
    traits::add::{AddCapability, CapTagAbsent},
};
use maud::Markup;

use crate::accounting_preferences_patch::{
    AccountingPreferencesRegistry, store_accounting_preferences_addons,
};

static LINKS: OnceLock<Vec<AccountingSidebarLink>> = OnceLock::new();

/// Capability tag for the accounting sidebar registry.
pub struct AccountingSidebarTag;

/// One navigation entry in the accounting app sidebar.
#[derive(Clone, Debug)]
pub struct AccountingSidebarLink {
    pub section: &'static str,
    pub label: &'static str,
    pub url: String,
    pub order: u16,
    pub icon: Option<&'static str>,
}

/// Build a sidebar link from a typed route tag (compile-time path checking).
pub fn link<R: RouteUrl + Default>(
    section: &'static str,
    label: &'static str,
    order: u16,
    icon: Option<&'static str>,
) -> AccountingSidebarLink {
    AccountingSidebarLink {
        section,
        label,
        url: R::default().url(),
        order,
        icon,
    }
}

/// Plugin hook for patching accounting sidebar links and preferences (mirrors Go menu/page patches).
pub trait AccountingSidebarRegistrar: Sized {
    fn register_accounting_sidebar(
        self,
        cap: AccountingSidebarRegistry,
    ) -> AccountingSidebarRegistry;

    fn register_accounting_preferences(
        self,
        cap: AccountingPreferencesRegistry,
    ) -> AccountingPreferencesRegistry {
        let _ = self;
        cap
    }
}

/// Sidebar link registry folded from base + addon hooks.
#[derive(Clone, Debug, Default)]
pub struct AccountingSidebarRegistry {
    links: Vec<AccountingSidebarLink>,
}

impl AccountingSidebarRegistry {
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    pub fn push(mut self, link: AccountingSidebarLink) -> Self {
        if !self.links.iter().any(|l| l.section == link.section) {
            self.links.push(link);
        }
        self
    }

    fn sorted_links(self) -> Vec<AccountingSidebarLink> {
        let mut links = self.links;
        links.sort_by_key(|l| (l.order, l.label));
        links
    }

    /// Sorted sidebar links (for tests and inspection).
    pub fn links(&self) -> &[AccountingSidebarLink] {
        &self.links
    }
}

/// Builder-phase accounting sidebar capability.
#[derive(Clone, Default)]
pub struct AccountingSidebarCap<Hooks> {
    pub hooks: Hooks,
    pub items: AccountingSidebarRegistry,
    pub preferences: AccountingPreferencesRegistry,
    _tag: PhantomData<fn() -> AccountingSidebarTag>,
}

impl<Hooks> AccountingSidebarCap<Hooks> {
    pub fn new() -> Self
    where
        Hooks: Default,
    {
        Self {
            hooks: Hooks::default(),
            items: AccountingSidebarRegistry::new(),
            preferences: AccountingPreferencesRegistry::new(),
            _tag: PhantomData,
        }
    }

    pub fn add_hook<HTag, H>(
        self,
        hook: H,
    ) -> AccountingSidebarCap<HCons<Tagged<HTag, H>, Hooks>> {
        AccountingSidebarCap {
            hooks: HCons {
                head: Tagged::new(hook),
                tail: self.hooks,
            },
            items: self.items,
            preferences: self.preferences,
            _tag: PhantomData,
        }
    }

    /// Eagerly fold registrar hooks into items (testing / pre-mount inspection).
    pub fn resolve_hooks(self) -> AccountingSidebarCap<HNil>
    where
        Hooks: FoldSidebarRegistrarHooks,
    {
        let (items, preferences) = self.hooks.fold(self.items, self.preferences);
        AccountingSidebarCap {
            hooks: HNil,
            items,
            preferences,
            _tag: PhantomData,
        }
    }
}

impl<Hooks> HasCapTag for AccountingSidebarCap<Hooks> {
    type Tag = AccountingSidebarTag;
}

impl<Hooks, Plugin, Hook> CapHookExt<Plugin, Hook> for AccountingSidebarCap<Hooks> {
    type Hooked = AccountingSidebarCap<HCons<Tagged<Plugin, Hook>, Hooks>>;

    fn prepend_cap_hook(self, hook: Hook) -> Self::Hooked {
        self.add_hook::<Plugin, Hook>(hook)
    }
}

/// Fold registrar hooks over the sidebar and preferences registries (tail first = install order).
pub trait FoldSidebarRegistrarHooks {
    fn fold(
        self,
        sidebar: AccountingSidebarRegistry,
        preferences: AccountingPreferencesRegistry,
    ) -> (AccountingSidebarRegistry, AccountingPreferencesRegistry);
}

impl FoldSidebarRegistrarHooks for HNil {
    fn fold(
        self,
        sidebar: AccountingSidebarRegistry,
        preferences: AccountingPreferencesRegistry,
    ) -> (AccountingSidebarRegistry, AccountingPreferencesRegistry) {
        (sidebar, preferences)
    }
}

impl<Plugin, H, Tail> FoldSidebarRegistrarHooks for HCons<Tagged<Plugin, H>, Tail>
where
    Tail: FoldSidebarRegistrarHooks,
    H: AccountingSidebarRegistrar + Copy,
{
    fn fold(
        self,
        sidebar: AccountingSidebarRegistry,
        preferences: AccountingPreferencesRegistry,
    ) -> (AccountingSidebarRegistry, AccountingPreferencesRegistry) {
        let (sidebar, preferences) = self.tail.fold(sidebar, preferences);
        let hook = self.head.value;
        (
            hook.register_accounting_sidebar(sidebar),
            hook.register_accounting_preferences(preferences),
        )
    }
}

impl<Hooks> Capability for AccountingSidebarCap<Hooks>
where
    Hooks: FoldSidebarRegistrarHooks,
{
    type Value = AccountingSidebarRegistry;
    type Output = Tagged<AccountingSidebarTag, AccountingSidebarRegistry>;
    type Hooks = Hooks;
    type Items = AccountingSidebarRegistry;

    fn mount(self) -> Self::Output {
        let (registry, preferences) = self.hooks.fold(self.items, self.preferences);
        let sorted = registry.clone().sorted_links();
        let _ = LINKS.set(sorted);
        store_accounting_preferences_addons(&preferences);
        Tagged::new(registry)
    }
}

/// Attach an empty accounting sidebar capability to the app builder.
///
/// Prefer `cap_attach` in the accounts plugin install steps; this helper remains for
/// manual wiring or tests outside the install macro.
pub fn with_accounting_sidebar<L, Proof>(app: App<L>) -> App<HCons<AccountingSidebarCap<HNil>, L>>
where
    L: HList + CapTagAbsent<AccountingSidebarTag, Proof>,
{
    app.add_capability(AccountingSidebarCap::new())
}

/// Render the patched accounting sidebar.
pub fn accounting_sidebar() -> Markup {
    let links = LINKS
        .get()
        .expect("accounting sidebar not initialized — mount the app after finance plugins install");
    let back_url = DashboardAppsRouteTag.url();
    sidebar_menu(SidebarMenu {
        title: "Accounting",
        back: Some(SidebarMenuBack {
            title: "Back to Home",
            url: &back_url,
        }),
        children: {
            let mut items = Markup::default();
            for link in links {
                items = maud::html! {
                    (items)
                    (sidebar_menu_item(SidebarMenuItem {
                        title: link.label,
                        url: &link.url,
                        icon_name: link.icon,
                        ..Default::default()
                    }))
                };
            }
            items
        },
    })
}

/// Base accounting sidebar links registered by uniquity-finance-accounts.
#[derive(Clone, Copy, Default)]
pub struct BaseHook;

impl AccountingSidebarRegistrar for BaseHook {
    fn register_accounting_sidebar(
        self,
        cap: AccountingSidebarRegistry,
    ) -> AccountingSidebarRegistry {
        use crate::routes::{
            AccountingPreferencesRouteTag, CurrencyListRouteTag, FinanceDefaultRouteTag,
            JournalListRouteTag,
        };

        cap.push(link::<FinanceDefaultRouteTag>("accounts", "Accounts", 10, Some("building-library")))
            .push(link::<CurrencyListRouteTag>(
                "currencies",
                "Currencies",
                20,
                Some("currency-dollar"),
            ))
            .push(link::<JournalListRouteTag>("journals", "Journals", 30, Some("book-open")))
            .push(link::<AccountingPreferencesRouteTag>(
                "preferences",
                "Accounting preferences",
                40,
                Some("adjustments-horizontal"),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::FinanceDefaultRouteTag;

    struct TestAddonTag;

    #[derive(Copy, Clone)]
    struct TestAddonHook;

    impl AccountingSidebarRegistrar for TestAddonHook {
        fn register_accounting_sidebar(
            self,
            cap: AccountingSidebarRegistry,
        ) -> AccountingSidebarRegistry {
            cap.push(link::<FinanceDefaultRouteTag>(
                "test-addon",
                "Test addon",
                15,
                None,
            ))
        }
    }

    #[test]
    fn resolve_hooks_folds_base_and_addon_links() {
        let cap = AccountingSidebarCap::<HNil>::new()
            .add_hook::<crate::UniquityFinanceAccountsTag, _>(BaseHook)
            .add_hook::<TestAddonTag, _>(TestAddonHook)
            .resolve_hooks();

        assert_eq!(cap.items.links().len(), 5);
        let sections: Vec<_> = cap
            .items
            .links()
            .iter()
            .map(|l| l.section)
            .collect();
        assert!(sections.contains(&"accounts"));
        assert!(sections.contains(&"test-addon"));
        assert!(sections.contains(&"preferences"));
    }
}
