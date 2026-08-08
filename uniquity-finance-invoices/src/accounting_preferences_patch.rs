//! Patches invoice presentation + GL preferences onto `/finance/preferences`.

use std::collections::HashMap;

use chrono::Utc;
use lariv_rs::components::{
    CodeEditorInput, code_editor_input,
    attrs::escape_attr,
    htmx::{HTMX_SWAP_BODY_MODAL, HTMX_TARGET_BODY_MODAL},
    label_newline_hint,
};
use lariv_rs::html_form::FormFieldKey;
use maud::{Markup, PreEscaped, html};
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
        InvoicePreferencesForm, InvoicePreferencesFormField, InvoicePresentationPreferencesFormField,
        PaymentPreferencesForm, PaymentPreferencesFormField,
    },
    invoice_pdf_template::DEFAULT_INVOICE_PDF_TEMPLATE,
    logic::preferences::{load_invoice_preferences, load_payment_preferences},
    preferences_hints::{INVOICE_NUMBER_FORMAT_HINT, INVOICE_PDF_TEMPLATE_HINT},
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

fn param_opt_str(params: &HashMap<String, String>, key: &str) -> Option<String> {
    params.get(key).and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.clone())
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

        let number_format = inv.invoice_number_format.unwrap_or_default();
        let pdf_template = inv.invoice_pdf_template.unwrap_or_default();

        html! {
            (label_newline_hint(
                "Invoice number format",
                Some(INVOICE_NUMBER_FORMAT_HINT),
                html! {
                    input type="text"
                        name=(InvoicePresentationPreferencesFormField::InvoiceNumberFormat.html_name())
                        class="input input-bordered w-full"
                        value=(number_format) {}
                },
            ))
            (label_newline_hint(
                "Invoice PDF template (Typst + Minijinja)",
                Some(INVOICE_PDF_TEMPLATE_HINT),
                html! {
                    (code_editor_input(CodeEditorInput {
                        label: "",
                        name: InvoicePresentationPreferencesFormField::InvoicePdfTemplate.html_name(),
                        value: &pdf_template,
                        id: "invoice-pdf-template-field",
                        language: "plaintext",
                        rows: 16,
                        max_height: "24rem",
                        ..Default::default()
                    }))
                    textarea id="default-invoice-pdf-template" hidden readonly {
                        (DEFAULT_INVOICE_PDF_TEMPLATE)
                    }
                    div class="flex justify-end gap-2 mt-2" {
                        button type="button" class="btn btn-ghost btn-sm"
                            onclick="if (confirm('This will overwrite the template in the field with the default example template. Continue?')) { const ta = document.getElementById('invoice-pdf-template-field'); const def = document.getElementById('default-invoice-pdf-template'); if (!ta || !def) return; ta.value = def.value; const root = ta.closest('[data-code-editor-root]'); if (root) { root.dispatchEvent(new CustomEvent('code-editor:set', { detail: { value: def.value } })); } else { ta.dispatchEvent(new Event('change', { bubbles: true })); } }" {
                            "Use default template"
                        }
                        div class="fk-modal-host" {
                            (PreEscaped(format!(
                                r#"<button type="button" class="btn btn-outline btn-sm" hx-post="/finance-invoices/invoice-pdf-preview" hx-target="{}" hx-swap="{}" hx-include="closest form" hx-push-url="false">Preview sample PDF</button>"#,
                                escape_attr(HTMX_TARGET_BODY_MODAL),
                                escape_attr(HTMX_SWAP_BODY_MODAL),
                            )))
                        }
                    }
                },
            ))
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
        inv_am.invoice_number_format = Set(param_opt_str(
            params,
            InvoicePresentationPreferencesFormField::InvoiceNumberFormat.html_name(),
        ));
        inv_am.invoice_pdf_template = Set(param_opt_str(
            params,
            InvoicePresentationPreferencesFormField::InvoicePdfTemplate.html_name(),
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
