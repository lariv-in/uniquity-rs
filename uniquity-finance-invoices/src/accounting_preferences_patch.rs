//! Patches invoice and payment GL preferences onto `/finance/preferences`.

use std::collections::HashMap;

use chrono::Utc;
use lariv_rs::html_form::FormFieldKey;
use maud::{Markup, html};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use uniquity_finance_accounts::{
    account_select_route_url,
    accounting_preferences_patch::AccountingPreferencesAddon,
    logic::journal::{credit_balance_type, debit_balance_type},
    scope::{load_account_parent_label, load_journal_display_label},
};
use uniquity_finance_products::preferences::optional_i64;

use crate::{
    entities::{
        payment_preferences::{self},
        preferences::{self},
    },
    forms::{
        InvoicePreferencesForm, InvoicePreferencesFormField, PaymentPreferencesForm,
        PaymentPreferencesFormField,
    },
    logic::preferences::{load_invoice_preferences, load_payment_preferences},
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

pub(crate) struct InvoicesAccountingPreferencesAddon;

#[async_trait::async_trait]
impl AccountingPreferencesAddon for InvoicesAccountingPreferencesAddon {
    fn id(&self) -> &'static str {
        "finance-invoices"
    }

    async fn render_inputs(&self, db: &DatabaseConnection) -> Markup {
        use lariv_rs::html_form::{FormCtx, HtmlForm};

        let inv = load_invoice_preferences(db).await;
        let pay = load_payment_preferences(db).await;

        let ar_display =
            load_account_parent_label(db, inv.account_receivable_id).await;
        let revenue_display = load_account_parent_label(db, inv.account_revenue_id).await;
        let tax_display =
            load_account_parent_label(db, inv.account_tax_payable_id).await;
        let journal_display = load_journal_display_label(db, inv.journal_id).await;
        let payment_display =
            load_account_parent_label(db, pay.payment_account_id).await;

        let debit_url = account_select_route_url(debit_balance_type().as_str());
        let credit_url = account_select_route_url(credit_balance_type().as_str());

        html! {
            (InvoicePreferencesForm::render_inputs(
                &FormCtx::form::<InvoicePreferencesForm>()
                    .value(
                        InvoicePreferencesFormField::AccountReceivableId,
                        fk_value(inv.account_receivable_id),
                    )
                    .display(InvoicePreferencesFormField::AccountReceivableId, &ar_display)
                    .url(InvoicePreferencesFormField::AccountReceivableId, &debit_url)
                    .value(
                        InvoicePreferencesFormField::AccountRevenueId,
                        fk_value(inv.account_revenue_id),
                    )
                    .display(InvoicePreferencesFormField::AccountRevenueId, &revenue_display)
                    .url(InvoicePreferencesFormField::AccountRevenueId, &credit_url)
                    .value(
                        InvoicePreferencesFormField::AccountTaxPayableId,
                        fk_value(inv.account_tax_payable_id),
                    )
                    .display(
                        InvoicePreferencesFormField::AccountTaxPayableId,
                        &tax_display,
                    )
                    .url(InvoicePreferencesFormField::AccountTaxPayableId, &credit_url)
                    .value(
                        InvoicePreferencesFormField::JournalId,
                        fk_value(inv.journal_id),
                    )
                    .display(InvoicePreferencesFormField::JournalId, &journal_display),
            ))
            (PaymentPreferencesForm::render_inputs(
                &FormCtx::form::<PaymentPreferencesForm>()
                    .value(
                        PaymentPreferencesFormField::PaymentAccountId,
                        fk_value(pay.payment_account_id),
                    )
                    .display(PaymentPreferencesFormField::PaymentAccountId, &payment_display)
                    .url(PaymentPreferencesFormField::PaymentAccountId, &debit_url),
            ))
        }
    }

    async fn save_from_form(
        &self,
        db: &DatabaseConnection,
        params: &HashMap<String, String>,
    ) -> Result<(), String> {
        let now = Utc::now();

        let inv_prefs = load_invoice_preferences(db).await;
        let mut inv_am: preferences::ActiveModel = inv_prefs.into();
        inv_am.account_receivable_id = Set(param_opt_i64(
            params,
            InvoicePreferencesFormField::AccountReceivableId.html_name(),
        ));
        inv_am.account_revenue_id = Set(param_opt_i64(
            params,
            InvoicePreferencesFormField::AccountRevenueId.html_name(),
        ));
        inv_am.account_tax_payable_id = Set(param_opt_i64(
            params,
            InvoicePreferencesFormField::AccountTaxPayableId.html_name(),
        ));
        inv_am.journal_id = Set(param_opt_i64(
            params,
            InvoicePreferencesFormField::JournalId.html_name(),
        ));
        inv_am.updated_at = Set(Some(now));
        inv_am
            .update(db)
            .await
            .map_err(|e| e.to_string())?;

        let pay_prefs = load_payment_preferences(db).await;
        let mut pay_am: payment_preferences::ActiveModel = pay_prefs.into();
        pay_am.payment_account_id = Set(param_opt_i64(
            params,
            PaymentPreferencesFormField::PaymentAccountId.html_name(),
        ));
        pay_am.updated_at = Set(Some(now));
        pay_am
            .update(db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

pub(crate) static INVOICES_ADDON: InvoicesAccountingPreferencesAddon = InvoicesAccountingPreferencesAddon;
