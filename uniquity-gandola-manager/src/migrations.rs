use sea_orm_migration::prelude::*;

use super::GandolaManagerTag;

mod m00001_create_gandola_manager;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m00001_create_gandola_manager::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: GandolaManagerTag;
    migrator: Migrator;
}
