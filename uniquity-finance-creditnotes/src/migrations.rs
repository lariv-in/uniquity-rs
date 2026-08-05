use sea_orm_migration::prelude::*;

mod m00001_create_credit_notes;

use crate::UniquityFinanceCreditnotesTag;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m00001_create_credit_notes::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: UniquityFinanceCreditnotesTag;
    migrator: Migrator;
}
