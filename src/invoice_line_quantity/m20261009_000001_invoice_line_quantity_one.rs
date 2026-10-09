use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const INVOICE_LINE_TABLES: [&str; 3] = [
    "draft_invoice_lines",
    "posted_invoice_lines",
    "cancelled_invoice_lines",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        for table in INVOICE_LINE_TABLES {
            conn.execute_unprepared(&format!("UPDATE {table} SET quantity = 1"))
                .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
