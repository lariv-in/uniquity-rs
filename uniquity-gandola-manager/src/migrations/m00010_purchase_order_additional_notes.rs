use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PurchaseOrders {
    Table,
    AdditionalNotes,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .add_column(ColumnDef::new(PurchaseOrders::AdditionalNotes).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .drop_column(PurchaseOrders::AdditionalNotes)
                    .to_owned(),
            )
            .await
    }
}
