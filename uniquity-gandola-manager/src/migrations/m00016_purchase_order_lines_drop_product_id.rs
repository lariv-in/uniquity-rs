use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn execute(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute(Statement::from_string(
            manager.get_connection().get_database_backend(),
            sql.to_string(),
        ))
        .await
        .map(|_| ())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_connection().get_database_backend() {
            sea_orm::DatabaseBackend::Postgres => {
                execute(
                    manager,
                    "ALTER TABLE purchase_order_lines DROP CONSTRAINT IF EXISTS fk_purchase_order_lines_product_id",
                )
                .await?;
                execute(
                    manager,
                    "ALTER TABLE purchase_order_lines DROP COLUMN IF EXISTS product_id",
                )
                .await
            }
            sea_orm::DatabaseBackend::Sqlite => {
                execute(
                    manager,
                    "ALTER TABLE purchase_order_lines DROP COLUMN IF EXISTS product_id",
                )
                .await
            }
            _ => Ok(()),
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("purchase_order_lines"))
                    .add_column(ColumnDef::new(Alias::new("product_id")).big_integer())
                    .to_owned(),
            )
            .await
    }
}
