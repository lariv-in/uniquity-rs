use sea_orm_migration::prelude::*;

use super::UniquityFinanceCustomerTag;

mod m00001_create_customers;
mod m00002_split_customer_address;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_customers::Migration),
            Box::new(m00002_split_customer_address::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: UniquityFinanceCustomerTag;
    migrator: Migrator;
}
