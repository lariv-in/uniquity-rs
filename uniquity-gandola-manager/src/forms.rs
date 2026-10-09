use lariv_core::html_form::{
    html_form,
    widgets::{Date, Password, Select, Text, Textarea},
};
use lariv_plugin_contacts::routes::CompanyFkSelectRouteTag;
use lariv_plugin_filesystem::routes::{VNodeFileSelectRouteTag, VNodeSelectRouteTag};
use lariv_plugin_finance_invoices::forms::PaymentTermLinesDraft;
use lariv_plugin_finance_invoices::routes::DraftInvoiceMultiSelectRouteTag;
use lariv_plugin_finance_products::routes::ProductFkSelectRouteTag;

use crate::po_line_editor::PurchaseOrderLinesDraft;
use crate::po_payment_term::PurchaseOrderPaymentTermLinesDraft;

use super::routes::{
    GandolaSelectRouteTag, PurchaseOrderSelectRouteTag, SiteFkSelectRouteTag, SiteSelectRouteTag,
};
use super::site_status::SiteStatus;

#[html_form]
pub struct GandolaForm {
    #[form(label = "Gandola Name", required, widget = Text)]
    pub name: String,

    #[form(
        label = "Sites",
        widget = ManyToMany,
        route = SiteSelectRouteTag,
        swap_key = "gandola-sites",
        placeholder = "Select sites…"
    )]
    pub sites: Vec<i64>,
}

#[html_form]
pub struct GandolaFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,
}

#[html_form]
pub struct SiteForm {
    #[form(label = "Site Name", required, widget = Text)]
    pub name: String,

    #[form(label = "Site ID", widget = Text)]
    pub site_id: String,

    #[form(
        label = "Customer",
        required,
        widget = ForeignKey,
        route = CompanyFkSelectRouteTag,
        swap_key = "gandola-site-customer",
        display = "customer",
        placeholder = "Select customer…"
    )]
    pub customer_id: i64,

    #[form(label = "Status", widget = Select)]
    pub status: String,

    #[form(label = "Start Date", widget = Date)]
    pub start_date: String,

    #[form(label = "End Date", widget = Date)]
    pub end_date: String,

    #[form(label = "Address", widget = Textarea)]
    pub address: String,

    #[form(label = "Remarks", widget = Textarea)]
    pub remarks: String,

    #[form(
        label = "Gandolas",
        widget = ManyToMany,
        route = GandolaSelectRouteTag,
        swap_key = "site-gandolas",
        placeholder = "Select gandolas…"
    )]
    pub gandolas: Vec<i64>,

    #[form(
        label = "Invoices",
        widget = ManyToMany,
        route = DraftInvoiceMultiSelectRouteTag,
        swap_key = "site-invoices",
        placeholder = "Select invoices…"
    )]
    pub invoices: Vec<i64>,

    #[form(
        label = "Purchase Orders",
        widget = ManyToMany,
        route = PurchaseOrderSelectRouteTag,
        swap_key = "site-purchase-orders",
        placeholder = "Select purchase orders…"
    )]
    pub purchase_orders: Vec<i64>,
}

impl SiteForm {
    pub fn status_choices() -> &'static [(&'static str, &'static str)] {
        SiteStatus::choices()
    }
}

#[html_form]
pub struct DraftInvoiceSitesForm {
    #[form(
        label = "Sites",
        widget = ManyToMany,
        route = SiteSelectRouteTag,
        swap_key = "invoice-sites",
        placeholder = "Select sites…"
    )]
    pub sites: Vec<i64>,
}

/// Sites column filter on the invoice hub. The HTML name matches the column key.
#[html_form]
pub struct InvoiceHubSitesFilterForm {
    #[form(
        label = "Sites",
        widget = ForeignKey,
        route = SiteFkSelectRouteTag,
        swap_key = "invoice-hub-site",
        display = "site",
        placeholder = "Select site…"
    )]
    pub sites: String,
}

#[html_form]
pub struct SiteInvoiceFilterForm {
    #[form(label = "Number", widget = Text, name = "InvoiceNumber")]
    pub number: String,

    #[form(label = "Status", widget = Select, name = "InvoiceStatus")]
    pub status: String,
}

impl SiteInvoiceFilterForm {
    pub fn status_choices() -> &'static [(&'static str, &'static str)] {
        &[
            ("", "Any"),
            ("Draft", "Draft"),
            ("Posted", "Posted"),
            ("Partial", "Partial"),
            ("Paid", "Paid"),
            ("Cancelled", "Cancelled"),
        ]
    }
}

#[html_form]
pub struct SiteFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,

    #[form(label = "Site ID", widget = Text)]
    pub site_id: String,
}

#[html_form]
pub struct GandolaPreferencesForm {
    #[form(
        label = "Gandola Rent Product",
        widget = ForeignKey,
        route = ProductFkSelectRouteTag,
        swap_key = "gandola-pref-rent-product",
        display = "gandola_product",
        placeholder = "Select product…"
    )]
    pub gandola_product_id: String,

    #[form(
        label = "TPI Product",
        widget = ForeignKey,
        route = ProductFkSelectRouteTag,
        swap_key = "gandola-pref-tpi-product",
        display = "tpi_product",
        placeholder = "Select product…"
    )]
    pub tpi_product_id: String,

    #[form(
        label = "DTI Product",
        widget = ForeignKey,
        route = ProductFkSelectRouteTag,
        swap_key = "gandola-pref-dti-product",
        display = "dti_product",
        placeholder = "Select product…"
    )]
    pub dti_product_id: String,

    #[form(
        label = "Purchase order files directory",
        widget = ForeignKey,
        route = VNodeSelectRouteTag,
        swap_key = "gandola-pref-po-files-dir",
        display = "po_files_directory",
        placeholder = "Select directory…"
    )]
    pub purchase_order_files_directory_id: String,

    #[form(label = "Gemini API key", widget = Password)]
    pub gemini_api_key: String,

    #[form(label = "Gemini model", widget = Select, required, choices = "gemini_model")]
    pub gemini_model: String,

    #[form(label = "Invoice payment term", widget = PaymentTermLinesDraft)]
    pub payment_term_lines_json: String,
}

#[html_form]
pub struct PurchaseOrderForm {
    #[form(label = "Number", required, widget = Text)]
    pub number: String,

    #[form(label = "Date", required, widget = Date)]
    pub date: String,

    #[form(
        label = "Customer",
        required,
        widget = ForeignKey,
        route = CompanyFkSelectRouteTag,
        swap_key = "gandola-po-customer",
        display = "customer",
        placeholder = "Select customer…"
    )]
    pub customer_id: i64,

    #[form(
        label = "Site",
        required,
        widget = ForeignKey,
        route = SiteFkSelectRouteTag,
        swap_key = "gandola-po-site",
        display = "site",
        placeholder = "Select site…"
    )]
    pub site_id: i64,

    #[form(
        label = "File",
        widget = ForeignKey,
        route = VNodeFileSelectRouteTag,
        swap_key = "gandola-po-file",
        display = "file",
        placeholder = "Select file…"
    )]
    pub file_id: String,

    #[form(label = "Payment terms", required, widget = PurchaseOrderPaymentTermLinesDraft)]
    pub payment_term_lines_json: String,

    #[form(label = "Lines", required, widget = PurchaseOrderLinesDraft)]
    pub po_lines_json: String,

    #[form(label = "Billing address", widget = Textarea)]
    pub billing_address: String,

    #[form(label = "Shipping address", widget = Textarea)]
    pub shipping_address: String,
}

impl Clone for PurchaseOrderForm {
    fn clone(&self) -> Self {
        Self {
            csrf: self.csrf.clone(),
            number: self.number.clone(),
            date: self.date.clone(),
            customer_id: self.customer_id,
            site_id: self.site_id,
            file_id: self.file_id.clone(),
            payment_term_lines_json: self.payment_term_lines_json.clone(),
            po_lines_json: self.po_lines_json.clone(),
            billing_address: self.billing_address.clone(),
            shipping_address: self.shipping_address.clone(),
        }
    }
}

#[html_form]
pub struct PurchaseOrderFilterForm {
    #[form(label = "Number", widget = Text)]
    pub number: String,
}
