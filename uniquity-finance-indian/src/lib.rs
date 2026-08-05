#![feature(impl_trait_in_assoc_type)]

//! Indian GST seed data, default general ledger, and default finance preferences.

pub mod migrations;

pub struct UniquityFinanceIndianTag;

lariv_rs::define_plugin_install! {
    plugin: UniquityFinanceIndianTag;
    steps: [
        migrations(migrations::Hook),
    ]
}
