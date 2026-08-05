use sea_orm_migration::prelude::*;

mod m00001_create_taxes;
mod m00002_seed_service_tax;
mod m00003_tax_type_account;

use crate::UniquityFinanceTaxesTag;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_taxes::Migration),
            Box::new(m00002_seed_service_tax::Migration),
            Box::new(m00003_tax_type_account::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: UniquityFinanceTaxesTag;
    migrator: Migrator;
}
