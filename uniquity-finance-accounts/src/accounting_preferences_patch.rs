//! Preference addon types and render/save helpers for the unified `/finance/preferences` page.
//!
//! Plugins register addons via [`AccountingSidebarRegistrar::register_accounting_preferences`]
//! on the shared accounting sidebar capability hook.

use std::collections::HashMap;
use std::sync::OnceLock;

use async_trait::async_trait;
use maud::Markup;
use sea_orm::DatabaseConnection;

static ADDONS: OnceLock<Vec<&'static dyn AccountingPreferencesAddon>> = OnceLock::new();

/// One plugin's extra fields on `/finance/preferences` (GET render + POST save).
#[async_trait]
pub trait AccountingPreferencesAddon: Send + Sync {
    fn id(&self) -> &'static str;
    async fn render_inputs(&self, db: &DatabaseConnection) -> Markup;
    async fn save_from_form(
        &self,
        db: &DatabaseConnection,
        params: &HashMap<String, String>,
    ) -> Result<(), String>;
}

/// Registry of preference addons folded from plugin hooks.
#[derive(Clone, Default)]
pub struct AccountingPreferencesRegistry {
    addons: Vec<&'static dyn AccountingPreferencesAddon>,
}

impl AccountingPreferencesRegistry {
    pub fn new() -> Self {
        Self {
            addons: Vec::new(),
        }
    }

    pub fn register_addon(mut self, addon: &'static dyn AccountingPreferencesAddon) -> Self {
        let id = addon.id();
        if !self.addons.iter().any(|a| a.id() == id) {
            self.addons.push(addon);
        }
        self
    }

    pub fn addons(&self) -> &[&'static dyn AccountingPreferencesAddon] {
        &self.addons
    }
}

pub(crate) fn store_accounting_preferences_addons(registry: &AccountingPreferencesRegistry) {
    let _ = ADDONS.set(registry.addons.clone());
}

/// Registered preference addons (empty until app mount).
pub fn accounting_preferences_addons() -> &'static [&'static dyn AccountingPreferencesAddon] {
    ADDONS.get().map(|v| v.as_slice()).unwrap_or(&[])
}

/// Render all patched preference form sections.
pub async fn render_accounting_preferences_addons(db: &DatabaseConnection) -> Markup {
    let mut out = Markup::default();
    for addon in accounting_preferences_addons() {
        let section = addon.render_inputs(db).await;
        out = maud::html! { (out) (section) };
    }
    out
}

/// Persist all patched preference sections from a urlencoded form body.
pub async fn save_accounting_preferences_addons(
    db: &DatabaseConnection,
    params: &HashMap<String, String>,
) -> Result<(), String> {
    for addon in accounting_preferences_addons() {
        addon.save_from_form(db, params).await?;
    }
    Ok(())
}
