use sea_orm_migration::prelude::*;

use super::GandolaManagerTag;

mod m00001_create_gandola_manager;
mod m00002_site_invoices;
mod m00003_rename_gandola_sites;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_gandola_manager::Migration),
            Box::new(m00002_site_invoices::Migration),
            Box::new(m00003_rename_gandola_sites::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: GandolaManagerTag;
    migrator: Migrator;
}
