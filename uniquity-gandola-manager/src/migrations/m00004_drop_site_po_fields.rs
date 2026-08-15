use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Sites {
    Table,
    PoRent,
    PoDti,
    PoTpi,
    PoExtn1,
    PoExtn2,
    PoExtn3,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sites::Table)
                    .drop_column(Sites::PoRent)
                    .drop_column(Sites::PoDti)
                    .drop_column(Sites::PoTpi)
                    .drop_column(Sites::PoExtn1)
                    .drop_column(Sites::PoExtn2)
                    .drop_column(Sites::PoExtn3)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sites::Table)
                    .add_column(ColumnDef::new(Sites::PoRent).text())
                    .add_column(ColumnDef::new(Sites::PoDti).text())
                    .add_column(ColumnDef::new(Sites::PoTpi).text())
                    .add_column(ColumnDef::new(Sites::PoExtn1).text())
                    .add_column(ColumnDef::new(Sites::PoExtn2).text())
                    .add_column(ColumnDef::new(Sites::PoExtn3).text())
                    .to_owned(),
            )
            .await
    }
}
