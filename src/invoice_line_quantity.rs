//! This deployment sets every finance invoice line quantity to 1.

use sea_orm_migration::prelude::*;

pub struct InvoiceLineQuantityTag;

mod m20261009_000001_invoice_line_quantity_one;

#[derive(Clone, Copy, Default)]
struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20261009_000001_invoice_line_quantity_one::Migration,
        )]
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
