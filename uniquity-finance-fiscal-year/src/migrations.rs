use sea_orm_migration::prelude::*;

use super::UniquityFinanceFiscalYearTag;

mod m00001_create_fiscal_years;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m00001_create_fiscal_years::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: UniquityFinanceFiscalYearTag;
    migrator: Migrator;
}
