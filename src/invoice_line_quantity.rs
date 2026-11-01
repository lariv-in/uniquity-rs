//! Invoice line quantity repairs for this deployment.
//!
//! `m20261009_000001_invoice_line_quantity_one` incorrectly forced every line
//! quantity to 1 and must stay registered (already applied). Later migrations
//! restore historical quantities where `pre_tax_amount / rate` still implies them.

use sea_orm_migration::prelude::*;

pub struct InvoiceLineQuantityTag;

mod m20261009_000001_invoice_line_quantity_one;
mod m20261031_000001_restore_invoice_line_quantity_from_pre_tax;

#[derive(Clone, Copy, Default)]
struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261009_000001_invoice_line_quantity_one::Migration),
            Box::new(m20261031_000001_restore_invoice_line_quantity_from_pre_tax::Migration),
        ]
    }
}

lariv_core::define_register_migrations! {
    plugin: InvoiceLineQuantityTag;
    migrator: Migrator;
}

lariv_core::define_plugin_install! {
    plugin: InvoiceLineQuantityTag;
    steps: [
        migrations(Hook),
    ]
}
