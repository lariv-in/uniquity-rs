use lariv_rs::html_form::{
    FieldRender, FormCtx, FormWidget,
    html_form,
    widgets::{Datetime, Duration, ForeignKey, ManyToMany, Select, Text, Textarea},
};
use maud::Markup;

use uniquity_finance_accounts::routes::{AccountSelectRouteTag, JournalSelectRouteTag};
use uniquity_finance_customer::routes::CustomerFkSelectRouteTag;
use uniquity_finance_taxes::routes::TaxMultiSelectRouteTag;

use crate::components::{InputInvoiceLinesDraft, input_invoice_lines_draft};
use crate::entities::payment_term::{PAYMENT_TERM_TYPE_DUE_DATE, PAYMENT_TERM_TYPE_RELATIVE};
use crate::routes::{PaymentTermFkSelectRouteTag, PostedInvoiceFkSelectRouteTag};

/// Custom widget for draft invoice lines (Alpine editor + hidden JSON).
pub struct InvoiceLinesDraft;
impl FormWidget for InvoiceLinesDraft {
    fn render(ctx: &FormCtx<'_>, field: &FieldRender<'_>) -> Markup {
        input_invoice_lines_draft(InputInvoiceLinesDraft {
            name: field.name,
            defaults: field.value,
            preview: ctx.display_of("invoice_lines_preview"),
            ..Default::default()
        })
    }
}

#[html_form]
pub struct DraftInvoiceForm {
    #[form(label = "Number (optional)", widget = Text)]
    pub number: String,

    #[form(label = "Reference (optional)", widget = Text)]
    pub reference: String,

    #[form(label = "Payment reference (optional)", widget = Text)]
    pub payment_reference: String,

    #[form(label = "Bank account (optional)", widget = Text)]
    pub bank_account: String,

    #[form(label = "Datetime", required, widget = Datetime)]
    pub datetime: String,

    #[form(
        label = "Customer",
        required,
        widget = ForeignKey,
        route = CustomerFkSelectRouteTag,
        swap_key = "fk-invoice-customer",
        display = "customer",
        placeholder = "Select customer…"
    )]
    pub customer_id: i64,

    #[form(
        label = "Payment term",
        required,
        widget = ForeignKey,
        route = PaymentTermFkSelectRouteTag,
        swap_key = "fk-invoice-payment-term",
        display = "payment_term",
        placeholder = "Select payment term…"
    )]
    pub payment_term_id: i64,

    #[form(
        label = "Taxes",
        widget = ManyToMany,
        route = TaxMultiSelectRouteTag,
        swap_key = "invoice-header-taxes",
        placeholder = "Select taxes…"
    )]
    pub taxes: Vec<i64>,

    #[form(label = "Lines", required, widget = InvoiceLinesDraft, display = "invoice_lines_preview")]
    pub invoice_lines_json: String,
}

#[html_form]
pub struct PaymentTermForm {
    #[form(label = "Kind", required, widget = Select, model = "paymentTermType")]
    pub term_type: String,

    #[form(
        label = "Due date & time",
        widget = Datetime,
        show = "paymentTermType === 'p_uniquity_finance_invoices.PaymentTermDueDate'"
    )]
    pub due_datetime: String,

    #[form(
        label = "Offset duration",
        widget = Duration,
        show = "paymentTermType === 'p_uniquity_finance_invoices.PaymentTermRelative'"
    )]
    pub duration: String,
}

impl PaymentTermForm {
    pub fn term_type_choices() -> &'static [(&'static str, &'static str)] {
        &[
            (PAYMENT_TERM_TYPE_DUE_DATE, "Due on calendar date"),
            (PAYMENT_TERM_TYPE_RELATIVE, "Relative"),
        ]
    }

    /// Alpine `x-data` for the create form (`paymentTermType` drives conditional fields).
    pub fn alpine_x_data(term_type: &str) -> String {
        format!(
            "{{ paymentTermType: {} }}",
            serde_json::to_string(term_type).unwrap_or_else(|_| "\"\"".to_string())
        )
    }
}

#[html_form]
pub struct PaymentForm {
    #[form(
        label = "Posted invoice",
        required,
        widget = ForeignKey,
        route = PostedInvoiceFkSelectRouteTag,
        swap_key = "posted-invoice-select",
        display = "posted_invoice",
        placeholder = "Select posted invoice…"
    )]
    pub posted_invoice_id: i64,

    #[form(label = "Settlement amount", required, widget = Text)]
    pub amount: String,

    #[form(
        label = "Payment account (optional)",
        widget = ForeignKey,
        route = AccountSelectRouteTag,
        swap_key = "payment-account",
        display = "payment_account",
        placeholder = "Uses preference default…"
    )]
    pub account_id: String,

    #[form(label = "Payment date & time", required, widget = Datetime)]
    pub datetime: String,

    #[form(
        label = "Withholding taxes",
        widget = ManyToMany,
        route = TaxMultiSelectRouteTag,
        swap_key = "payment-withholding-taxes",
        placeholder = "Optional withholding at collection…"
    )]
    pub taxes: Vec<i64>,
}

/// Header fields for batch payment (allocations use a custom JSON editor widget).
#[html_form]
pub struct PaymentBatchForm {
    #[form(label = "Payment date & time", required, widget = Datetime)]
    pub datetime: String,

    #[form(
        label = "Payment account (optional)",
        widget = ForeignKey,
        route = AccountSelectRouteTag,
        swap_key = "payment-batch-account",
        display = "payment_account",
        placeholder = "Uses preference default…"
    )]
    pub account_id: String,

    #[form(label = "Allocations", required, widget = PaymentBatchAllocations, display = "batch_allocations_preview")]
    pub allocations_json: String,
}

/// Custom widget for batch payment allocations (Alpine editor + hidden JSON).
pub struct PaymentBatchAllocations;
impl FormWidget for PaymentBatchAllocations {
    fn render(ctx: &FormCtx<'_>, field: &FieldRender<'_>) -> Markup {
        let preview = ctx.display_of("batch_allocations_preview");
        #[derive(serde::Deserialize, Default)]
        struct Preview {
            #[serde(default)]
            tax_pct_by_id: serde_json::Map<String, serde_json::Value>,
            #[serde(default)]
            all_taxes: Vec<serde_json::Value>,
        }
        let parsed: Preview = serde_json::from_str(preview).unwrap_or_default();
        let tax_pct_json =
            serde_json::to_string(&parsed.tax_pct_by_id).unwrap_or_else(|_| "{}".into());
        let all_taxes_json =
            serde_json::to_string(&parsed.all_taxes).unwrap_or_else(|_| "[]".into());
        crate::components::input_payment_batch_allocations(
            crate::components::InputPaymentBatchAllocations {
                name: field.name,
                defaults: field.value,
                tax_pct_json: &tax_pct_json,
                all_taxes_json: &all_taxes_json,
                ..Default::default()
            },
        )
    }
}

#[html_form]
pub struct InvoicePreferencesForm {
    #[form(
        label = "Accounts receivable (invoices)",
        widget = ForeignKey,
        route = AccountSelectRouteTag,
        swap_key = "pref-invoice-ar",
        display = "account_receivable",
        placeholder = "Select debit account…"
    )]
    pub account_receivable_id: String,

    #[form(
        label = "Revenue account (invoices)",
        widget = ForeignKey,
        route = AccountSelectRouteTag,
        swap_key = "pref-invoice-revenue",
        display = "account_revenue",
        placeholder = "Select credit account…"
    )]
    pub account_revenue_id: String,

    #[form(
        label = "Tax payable (invoices)",
        widget = ForeignKey,
        route = AccountSelectRouteTag,
        swap_key = "pref-invoice-tax",
        display = "account_tax_payable",
        placeholder = "Select credit account…"
    )]
    pub account_tax_payable_id: String,

    #[form(
        label = "Journal (invoices)",
        widget = ForeignKey,
        route = JournalSelectRouteTag,
        swap_key = "pref-invoice-journal",
        display = "journal",
        placeholder = "Select journal…"
    )]
    pub journal_id: String,
}

#[html_form]
pub struct PaymentPreferencesForm {
    #[form(
        label = "Payment account (receipts)",
        widget = ForeignKey,
        route = AccountSelectRouteTag,
        swap_key = "pref-payment-account",
        display = "payment_account",
        placeholder = "Bank or cash account…"
    )]
    pub payment_account_id: String,
}

#[html_form]
pub struct CancelInvoiceForm {
    #[form(label = "Reason", required, widget = Textarea, rows = 3)]
    pub reason: String,
}
