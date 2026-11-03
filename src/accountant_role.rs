//! Deployment role `accountant`: Accounting, Gandola Manager, contacts, CRM,
//! the filesystem, and the assistant.
//! Customers live under Accounting and Gandola Manager (no separate customers app).
//! `accountant` can open the filesystem and the assistant chat.
//! Changing filesystem permissions stays with admin and superuser.
//! Changing assistant skills stays with admin and superuser. Assistant preferences stay superuser-only.

use lariv_core::apps::{AppsCapability, AppsRegistrar};
use lariv_plugin_contacts::routes::{ContactsMutate, ContactsView};
use lariv_plugin_crm::routes::{CrmMutate, CrmView};
use lariv_plugin_customer::routes::CustomerMutate;
use lariv_plugin_filesystem::FILESYSTEM_APP_KEY;
use lariv_plugin_finance_accounts::{
    ACCOUNTING_APP_KEY,
    routes::{FinanceAccountsMutate, FinanceAccountsView},
};
use lariv_plugin_finance_creditnotes::routes::FinanceCreditNotesView;
use lariv_plugin_finance_invoices::routes::{FinanceInvoicesMutate, FinanceInvoicesView};
use lariv_plugin_finance_payments::routes::{FinancePaymentsMutate, FinancePaymentsView};
use lariv_plugin_finance_products::routes::{FinanceProductsMutate, FinanceProductsView};
use lariv_plugin_finance_purchases::routes::{FinancePurchasesMutate, FinancePurchasesView};
use lariv_plugin_finance_taxes::routes::{FinanceTaxesMutate, FinanceTaxesView};
use lariv_plugin_llm_assistant::apps::{LLM_ASSISTANT_APP_KEY, allow_sidebar_role};
use lariv_plugin_users::{
    role_authorization::{RoleAuthorizationRegistrar, RoleAuthorizationRegistry},
    role_registry::{Role, RoleRegistrar, RoleRegistry},
};

use uniquity_gandola_manager::routes::Accountant;

pub const ACCOUNTANT_ROLE: &str = <Accountant as Role>::NAME;

const CONTACTS_APP_KEY: &str = "p_contacts";
const CRM_APP_KEY: &str = "p_crm";

pub struct AccountantRoleTag;

mod migrations;

lariv_core::define_plugin_install! {
    plugin: AccountantRoleTag;
    steps: [
        migrations(migrations::Hook),
        apps(AppsHook),
        cap_hook(lariv_plugin_users::role_authorization::RoleAuthorizationTag, lariv_plugin_users::role_authorization::RoleAuthorizationCap, RoleHook),
        cap_hook(lariv_plugin_users::role_registry::RoleRegistryTag, lariv_plugin_users::role_registry::RoleRegistryCap, CatalogHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct AppsHook;

impl AppsRegistrar for AppsHook {
    fn register_apps(self, apps: AppsCapability) -> AppsCapability {
        let apps = allow_accountant_app(apps, ACCOUNTING_APP_KEY);
        let apps = allow_accountant_app(apps, CONTACTS_APP_KEY);
        let apps = allow_accountant_app(apps, CRM_APP_KEY);
        let apps = allow_accountant_app(apps, FILESYSTEM_APP_KEY);
        allow_sidebar_role(ACCOUNTANT_ROLE);
        allow_accountant_app(apps, LLM_ASSISTANT_APP_KEY)
    }
}

fn allow_accountant_app(apps: AppsCapability, key: &str) -> AppsCapability {
    let Some(mut tile) = apps.apps().iter().find(|tile| tile.key == key).cloned() else {
        return apps;
    };
    if !tile.roles.iter().any(|role| role == ACCOUNTANT_ROLE) {
        tile.roles.push(ACCOUNTANT_ROLE.into());
    }
    apps.register(tile)
}

#[derive(Clone, Copy, Default)]
pub struct RoleHook;

impl RoleAuthorizationRegistrar for RoleHook {
    fn register_roles(self, registry: RoleAuthorizationRegistry) -> RoleAuthorizationRegistry {
        registry
            .patch::<FinanceAccountsView>(allow_accountant)
            .patch::<FinanceAccountsMutate>(allow_accountant)
            // FinanceAccountsPreferencesMutate stays superuser-only.
            .patch::<FinanceInvoicesView>(allow_accountant)
            .patch::<FinanceInvoicesMutate>(allow_accountant)
            .patch::<FinancePaymentsView>(allow_accountant)
            .patch::<FinancePaymentsMutate>(allow_accountant)
            .patch::<FinanceProductsView>(allow_accountant)
            .patch::<FinanceProductsMutate>(allow_accountant)
            .patch::<FinancePurchasesView>(allow_accountant)
            .patch::<FinancePurchasesMutate>(allow_accountant)
            .patch::<FinanceTaxesView>(allow_accountant)
            .patch::<FinanceTaxesMutate>(allow_accountant)
            .patch::<FinanceCreditNotesView>(allow_accountant)
            .patch::<CustomerMutate>(allow_accountant)
            .patch::<ContactsView>(allow_accountant)
            .patch::<ContactsMutate>(allow_accountant)
            .patch::<CrmView>(allow_accountant)
            .patch::<CrmMutate>(allow_accountant)
        // FilesystemPermissions stays admin and superuser.
        // LlmSkillsMutate stays admin and superuser.
        // LlmPrefsAdmin stays superuser-only.
    }
}

fn allow_accountant(roles: &mut Vec<String>) {
    if !roles.iter().any(|role| role == ACCOUNTANT_ROLE) {
        roles.push(ACCOUNTANT_ROLE.into());
    }
}

/// Registers [`Accountant`] on the role catalog so it can be assigned to a user.
#[derive(Clone, Copy, Default)]
pub struct CatalogHook;

impl RoleRegistrar for CatalogHook {
    fn register_roles(self, registry: RoleRegistry) -> RoleRegistry {
        registry.register::<Accountant>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lariv_core::apps::{AppsCapability, AppsRegistrar};
    use lariv_plugin_contacts::apps::Hook as ContactsAppsHook;
    use lariv_plugin_filesystem::{
        apps::Hook as FilesystemAppsHook,
        routes::{FilesystemPermissions, RoleHook as FilesystemRoleHook},
    };
    use lariv_plugin_llm_assistant::{
        apps::Hook as LlmAppsHook,
        routes::{LlmPrefsAdmin, LlmSkillsMutate, RoleHook as LlmRoleHook},
    };
    use lariv_plugin_users::role_authorization::RoleAuthorizationRegistrar;

    #[test]
    fn accountant_can_open_filesystem_but_not_change_permissions() {
        let apps = FilesystemAppsHook.register_apps(AppsCapability::new());
        let apps = AppsHook.register_apps(apps);
        let filesystem = apps
            .apps()
            .iter()
            .find(|tile| tile.key == FILESYSTEM_APP_KEY)
            .expect("filesystem tile");
        assert!(filesystem.roles.iter().any(|role| role == ACCOUNTANT_ROLE));
        assert!(
            apps.visible_apps(ACCOUNTANT_ROLE)
                .iter()
                .any(|tile| tile.key == FILESYSTEM_APP_KEY)
        );

        let registry = FilesystemRoleHook.register_roles(RoleAuthorizationRegistry::new());
        let registry = RoleHook.register_roles(registry);
        let permissions = registry.roles::<FilesystemPermissions>();
        assert!(permissions.iter().any(|role| role == "admin"));
        assert!(!permissions.iter().any(|role| role == ACCOUNTANT_ROLE));
    }

    #[test]
    fn accountant_can_open_assistant_but_not_change_skills_or_preferences() {
        let apps = LlmAppsHook.register_apps(AppsCapability::new());
        let apps = AppsHook.register_apps(apps);
        let assistant = apps
            .apps()
            .iter()
            .find(|tile| tile.key == LLM_ASSISTANT_APP_KEY)
            .expect("assistant tile");
        assert!(assistant.roles.iter().any(|role| role == ACCOUNTANT_ROLE));
        assert!(
            apps.visible_apps(ACCOUNTANT_ROLE)
                .iter()
                .any(|tile| tile.key == LLM_ASSISTANT_APP_KEY)
        );
        assert!(lariv_plugin_llm_assistant::apps::sidebar_visible(Some(
            ACCOUNTANT_ROLE
        )));

        let registry = LlmRoleHook.register_roles(RoleAuthorizationRegistry::new());
        let registry = RoleHook.register_roles(registry);
        let skills = registry.roles::<LlmSkillsMutate>();
        assert!(skills.iter().any(|role| role == "admin"));
        assert!(!skills.iter().any(|role| role == ACCOUNTANT_ROLE));
        let preferences = registry.roles::<LlmPrefsAdmin>();
        assert!(preferences.is_empty());
        assert!(!preferences.iter().any(|role| role == ACCOUNTANT_ROLE));
    }

    #[test]
    fn accountant_can_open_contacts_and_crm_and_mutate_customers() {
        use lariv_plugin_contacts::routes::RoleHook as ContactsRoleHook;
        use lariv_plugin_crm::apps::Hook as CrmAppsHook;
        use lariv_plugin_crm::routes::RoleHook as CrmRoleHook;
        use lariv_plugin_customer::routes::RoleHook as CustomerRoleHook;

        let apps = ContactsAppsHook.register_apps(AppsCapability::new());
        let apps = CrmAppsHook.register_apps(apps);
        let apps = AppsHook.register_apps(apps);
        for key in [CONTACTS_APP_KEY, CRM_APP_KEY] {
            let tile = apps
                .apps()
                .iter()
                .find(|tile| tile.key == key)
                .expect("app tile");
            assert!(tile.roles.iter().any(|role| role == ACCOUNTANT_ROLE));
            assert!(
                apps.visible_apps(ACCOUNTANT_ROLE)
                    .iter()
                    .any(|tile| tile.key == key)
            );
        }

        let registry = CustomerRoleHook.register_roles(RoleAuthorizationRegistry::new());
        let registry = ContactsRoleHook.register_roles(registry);
        let registry = CrmRoleHook.register_roles(registry);
        let registry = RoleHook.register_roles(registry);
        for roles in [
            registry.roles::<CustomerMutate>(),
            registry.roles::<ContactsView>(),
            registry.roles::<ContactsMutate>(),
            registry.roles::<CrmView>(),
            registry.roles::<CrmMutate>(),
        ] {
            assert!(roles.iter().any(|role| role == ACCOUNTANT_ROLE));
        }
    }
}
