use sea_orm_migration::prelude::*;

use super::GandolaManagerTag;

mod m00001_create_gandola_manager;
mod m00002_site_invoices;
mod m00003_rename_gandola_sites;
mod m00004_drop_site_po_fields;
mod m00005_create_purchase_orders;
mod m00006_purchase_order_line_fields;
mod m00007_purchase_order_payment_term_fk;
mod m00008_purchase_order_payment_terms;
mod m00009_gandola_gemini_api_key;
mod m00010_purchase_order_additional_notes;
mod m00011_gandola_gemini_model;
mod m00012_purchase_order_drop_cin;
mod m00013_purchase_order_site_id;
mod m00014_purchase_order_drop_additional_notes;
mod m00015_pg_trgm;
mod m00016_purchase_order_lines_drop_product_id;
mod m00017_pg_trgm_lower;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_gandola_manager::Migration),
            Box::new(m00002_site_invoices::Migration),
            Box::new(m00003_rename_gandola_sites::Migration),
            Box::new(m00004_drop_site_po_fields::Migration),
            Box::new(m00005_create_purchase_orders::Migration),
            Box::new(m00006_purchase_order_line_fields::Migration),
            Box::new(m00007_purchase_order_payment_term_fk::Migration),
            Box::new(m00008_purchase_order_payment_terms::Migration),
            Box::new(m00009_gandola_gemini_api_key::Migration),
            Box::new(m00010_purchase_order_additional_notes::Migration),
            Box::new(m00011_gandola_gemini_model::Migration),
            Box::new(m00012_purchase_order_drop_cin::Migration),
            Box::new(m00013_purchase_order_site_id::Migration),
            Box::new(m00014_purchase_order_drop_additional_notes::Migration),
            Box::new(m00015_pg_trgm::Migration),
            Box::new(m00016_purchase_order_lines_drop_product_id::Migration),
            Box::new(m00017_pg_trgm_lower::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: GandolaManagerTag;
    migrator: Migrator;
}
