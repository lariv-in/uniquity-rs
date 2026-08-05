//! Patches product GL preferences onto `/finance/preferences`.

use std::collections::HashMap;

use chrono::Utc;
use lariv_rs::html_form::FormFieldKey;
use maud::Markup;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use uniquity_finance_accounts::{
    accounting_preferences_patch::AccountingPreferencesAddon,
    scope::load_account_parent_label,
};
use crate::preferences::{load_product_preferences, optional_i64};

use crate::{
    entities::product_preferences::{self},
    forms::{ProductPreferencesForm, ProductPreferencesFormField},
};

fn param_opt_i64(params: &HashMap<String, String>, key: &str) -> Option<i64> {
    params.get(key).and_then(|s| {
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            s.parse().ok()
        }
    })
}

fn fk_value(id: Option<i64>) -> String {
    optional_i64(id).to_string()
}

pub(crate) struct ProductsAccountingPreferencesAddon;

#[async_trait::async_trait]
impl AccountingPreferencesAddon for ProductsAccountingPreferencesAddon {
    fn id(&self) -> &'static str {
        "finance-products"
    }

    async fn render_inputs(&self, db: &DatabaseConnection) -> Markup {
        use lariv_rs::html_form::{FormCtx, HtmlForm};
        use maud::html;

        let prefs = load_product_preferences(db).await;
        let inventory_display =
            load_account_parent_label(db, prefs.inventory_account_id).await;
        let cos_display =
            load_account_parent_label(db, prefs.cost_of_sales_account_id).await;

        html! {
            (ProductPreferencesForm::render_inputs(
                &FormCtx::form::<ProductPreferencesForm>()
                    .value(
                        ProductPreferencesFormField::InventoryAccountId,
                        fk_value(prefs.inventory_account_id),
                    )
                    .display(
                        ProductPreferencesFormField::InventoryAccountId,
                        &inventory_display,
                    )
                    .value(
                        ProductPreferencesFormField::CostOfSalesAccountId,
                        fk_value(prefs.cost_of_sales_account_id),
                    )
                    .display(
                        ProductPreferencesFormField::CostOfSalesAccountId,
                        &cos_display,
                    ),
            ))
        }
    }

    async fn save_from_form(
        &self,
        db: &DatabaseConnection,
        params: &HashMap<String, String>,
    ) -> Result<(), String> {
        let prefs = load_product_preferences(db).await;
        let now = Utc::now();
        let mut am: product_preferences::ActiveModel = prefs.into();
        am.inventory_account_id = Set(param_opt_i64(
            params,
            ProductPreferencesFormField::InventoryAccountId.html_name(),
        ));
        am.cost_of_sales_account_id = Set(param_opt_i64(
            params,
            ProductPreferencesFormField::CostOfSalesAccountId.html_name(),
        ));
        am.updated_at = Set(Some(now));
        am.update(db).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub(crate) static PRODUCTS_ADDON: ProductsAccountingPreferencesAddon = ProductsAccountingPreferencesAddon;
