use lariv_rs::html_form::{
    html_form,
    widgets::{Date, Select, Text, Textarea},
};
use lariv_rs::plugins::customer::routes::CustomerFkSelectRouteTag;
use lariv_rs::plugins::finance_invoices::forms::PaymentTermLinesDraft;
use lariv_rs::plugins::finance_products::routes::ProductFkSelectRouteTag;

use super::routes::{GandolaSelectRouteTag, SiteSelectRouteTag};
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

    #[form(
        label = "Customer",
        required,
        widget = ForeignKey,
        route = CustomerFkSelectRouteTag,
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

    #[form(label = "PO Rent", widget = Text)]
    pub po_rent: String,

    #[form(label = "PO DTI", widget = Text)]
    pub po_dti: String,

    #[form(label = "PO TPI", widget = Text)]
    pub po_tpi: String,

    #[form(label = "PO Extension 1", widget = Text)]
    pub po_extn1: String,

    #[form(label = "PO Extension 2", widget = Text)]
    pub po_extn2: String,

    #[form(label = "PO Extension 3", widget = Text)]
    pub po_extn3: String,

    #[form(
        label = "Gandolas",
        widget = ManyToMany,
        route = GandolaSelectRouteTag,
        swap_key = "site-gandolas",
        placeholder = "Select gandolas…"
    )]
    pub gandolas: Vec<i64>,
}

impl SiteForm {
    pub fn status_choices() -> &'static [(&'static str, &'static str)] {
        SiteStatus::choices()
    }
}

#[html_form]
pub struct SiteFilterForm {
    #[form(label = "Name", widget = Text)]
    pub name: String,
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

    #[form(label = "Invoice payment term", widget = PaymentTermLinesDraft)]
    pub payment_term_lines_json: String,
}
