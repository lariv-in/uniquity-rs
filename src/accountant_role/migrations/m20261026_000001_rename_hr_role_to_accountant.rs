//! Rename the stored deployment role `hr` to `accountant`.

use lariv_core::db::migration_sql::exec_sql;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const ROLE_TABLES: [&str; 3] = ["users", "filesystem_nodes", "filesystem_preferences"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in ROLE_TABLES {
            exec_sql(
                manager,
                &format!("UPDATE {table} SET role = 'accountant' WHERE role = 'hr'"),
            )
            .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in ROLE_TABLES {
            exec_sql(
                manager,
                &format!("UPDATE {table} SET role = 'hr' WHERE role = 'accountant'"),
            )
            .await?;
        }
        Ok(())
    }
}
