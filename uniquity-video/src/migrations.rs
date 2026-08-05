use sea_orm_migration::prelude::*;

use super::UniquityVideoTag;

mod m20260803_000001_create_video;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260803_000001_create_video::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: UniquityVideoTag;
    migrator: Migrator;
}
