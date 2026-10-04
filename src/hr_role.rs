//! Deployment role `hr`: Accounting, Gandola Manager, the filesystem, and the assistant.
//! `hr` can open the filesystem and the assistant chat.
//! Changing filesystem permissions stays with admin and superuser.
//! Changing assistant skills stays with admin and superuser. Assistant preferences stay superuser-only.

use lariv_rs::{
    apps::{AppsCapability, AppsRegistrar},
    plugins::{
        customer::routes::{CustomerMutate, CustomerView},
        filesystem::FILESYSTEM_APP_KEY,
        finance_accounts::{
            ACCOUNTING_APP_KEY,
            routes::{FinanceAccountsMutate, FinanceAccountsView},
        },
        finance_creditnotes::routes::FinanceCreditNotesView,
        finance_invoices::routes::{FinanceInvoicesMutate, FinanceInvoicesView},
        finance_products::routes::{FinanceProductsMutate, FinanceProductsView},
        finance_taxes::routes::{FinanceTaxesMutate, FinanceTaxesView},
        llm_assistant::apps::{LLM_ASSISTANT_APP_KEY, allow_sidebar_role},
        users::{
            role_authorization::{RoleAuthorizationRegistrar, RoleAuthorizationRegistry},
            role_registry::{Role, RoleRegistrar, RoleRegistry},
        },
    },
};

use uniquity_gandola_manager::routes::Hr;

pub const HR_ROLE: &str = <Hr as Role>::NAME;

pub struct HrRoleTag;

lariv_rs::define_plugin_install! {
    plugin: HrRoleTag;
    steps: [
        apps(AppsHook),
        cap_hook(lariv_rs::plugins::users::role_authorization::RoleAuthorizationTag, lariv_rs::plugins::users::role_authorization::RoleAuthorizationCap, RoleHook),
        cap_hook(lariv_rs::plugins::users::role_registry::RoleRegistryTag, lariv_rs::plugins::users::role_registry::RoleRegistryCap, CatalogHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct AppsHook;

impl AppsRegistrar for AppsHook {
    fn register_apps(self, apps: AppsCapability) -> AppsCapability {
        let apps = allow_hr_app(apps, ACCOUNTING_APP_KEY);
        let apps = allow_hr_app(apps, FILESYSTEM_APP_KEY);
        allow_sidebar_role(HR_ROLE);
        allow_hr_app(apps, LLM_ASSISTANT_APP_KEY)
    }
}

fn allow_hr_app(apps: AppsCapability, key: &str) -> AppsCapability {
    let Some(mut tile) = apps.apps().iter().find(|tile| tile.key == key).cloned() else {
        return apps;
    };
    if !tile.roles.iter().any(|role| role == HR_ROLE) {
        tile.roles.push(HR_ROLE.into());
    }
    apps.register(tile)
}

#[derive(Clone, Copy, Default)]
pub struct RoleHook;

impl RoleAuthorizationRegistrar for RoleHook {
    fn register_roles(self, registry: RoleAuthorizationRegistry) -> RoleAuthorizationRegistry {
        registry
            .patch::<FinanceAccountsView>(allow_hr)
            .patch::<FinanceAccountsMutate>(allow_hr)
            // FinanceAccountsPreferencesMutate stays superuser-only.
            .patch::<FinanceInvoicesView>(allow_hr)
            .patch::<FinanceInvoicesMutate>(allow_hr)
            .patch::<FinanceProductsView>(allow_hr)
            .patch::<FinanceProductsMutate>(allow_hr)
            .patch::<FinanceTaxesView>(allow_hr)
            .patch::<FinanceTaxesMutate>(allow_hr)
            .patch::<FinanceCreditNotesView>(allow_hr)
            .patch::<CustomerView>(allow_hr)
            .patch::<CustomerMutate>(allow_hr)
        // FilesystemPermissions stays admin and superuser.
        // LlmSkillsMutate stays admin and superuser.
        // LlmPrefsAdmin stays superuser-only.
    }
}

fn allow_hr(roles: &mut Vec<String>) {
    if !roles.iter().any(|role| role == HR_ROLE) {
        roles.push(HR_ROLE.into());
    }
}

/// Registers [`Hr`] on the role catalog so it can be assigned to a user.
#[derive(Clone, Copy, Default)]
pub struct CatalogHook;

impl RoleRegistrar for CatalogHook {
    fn register_roles(self, registry: RoleRegistry) -> RoleRegistry {
        registry.register::<Hr>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lariv_rs::apps::{AppsCapability, AppsRegistrar};
    use lariv_rs::plugins::filesystem::{
        apps::Hook as FilesystemAppsHook,
        routes::{FilesystemPermissions, RoleHook as FilesystemRoleHook},
    };
    use lariv_rs::plugins::llm_assistant::{
        apps::Hook as LlmAppsHook,
        routes::{LlmPrefsAdmin, LlmSkillsMutate, RoleHook as LlmRoleHook},
    };
    use lariv_rs::plugins::users::role_authorization::RoleAuthorizationRegistrar;

    #[test]
    fn hr_can_open_filesystem_but_not_change_permissions() {
        let apps = FilesystemAppsHook.register_apps(AppsCapability::new());
        let apps = AppsHook.register_apps(apps);
        let filesystem = apps
            .apps()
            .iter()
            .find(|tile| tile.key == FILESYSTEM_APP_KEY)
            .expect("filesystem tile");
        assert!(filesystem.roles.iter().any(|role| role == HR_ROLE));
        assert!(
            apps.visible_apps(HR_ROLE)
                .iter()
                .any(|tile| tile.key == FILESYSTEM_APP_KEY)
        );

        let registry = FilesystemRoleHook.register_roles(RoleAuthorizationRegistry::new());
        let registry = RoleHook.register_roles(registry);
        let permissions = registry.roles::<FilesystemPermissions>();
        assert!(permissions.iter().any(|role| role == "admin"));
        assert!(!permissions.iter().any(|role| role == HR_ROLE));
    }

    #[test]
    fn hr_can_open_assistant_but_not_change_skills_or_preferences() {
        let apps = LlmAppsHook.register_apps(AppsCapability::new());
        let apps = AppsHook.register_apps(apps);
        let assistant = apps
            .apps()
            .iter()
            .find(|tile| tile.key == LLM_ASSISTANT_APP_KEY)
            .expect("assistant tile");
        assert!(assistant.roles.iter().any(|role| role == HR_ROLE));
        assert!(
            apps.visible_apps(HR_ROLE)
                .iter()
                .any(|tile| tile.key == LLM_ASSISTANT_APP_KEY)
        );
        assert!(lariv_rs::plugins::llm_assistant::apps::sidebar_visible(
            Some(HR_ROLE)
        ));

        let registry = LlmRoleHook.register_roles(RoleAuthorizationRegistry::new());
        let registry = RoleHook.register_roles(registry);
        let skills = registry.roles::<LlmSkillsMutate>();
        assert!(skills.iter().any(|role| role == "admin"));
        assert!(!skills.iter().any(|role| role == HR_ROLE));
        let preferences = registry.roles::<LlmPrefsAdmin>();
        assert!(preferences.is_empty());
        assert!(!preferences.iter().any(|role| role == HR_ROLE));
    }
}
