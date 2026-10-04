use sea_orm_migration::prelude::*;

use super::UniquityEmployeesTag;

mod m20260803_000001_create_employees;
mod m20260803_000002_points_superuser_trigger;
mod m20260803_000003_employees_drop_deleted_at;
mod m20261004_000001_points_from_user_role;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260803_000001_create_employees::Migration),
            Box::new(m20260803_000002_points_superuser_trigger::Migration),
            Box::new(m20260803_000003_employees_drop_deleted_at::Migration),
            Box::new(m20261004_000001_points_from_user_role::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: UniquityEmployeesTag;
    migrator: Migrator;
}
