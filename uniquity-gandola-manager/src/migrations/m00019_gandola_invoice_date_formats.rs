use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum GandolaPreferences {
    Table,
    InvoiceDateFormat,
    InvoiceDatetimeFormat,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .add_column(
                        ColumnDef::new(GandolaPreferences::InvoiceDateFormat)
                            .text()
                            .not_null()
                            .default("%d/%m/%Y"),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .add_column(
                        ColumnDef::new(GandolaPreferences::InvoiceDatetimeFormat)
                            .text()
                            .not_null()
                            .default("%d/%m/%Y %H:%M"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .drop_column(GandolaPreferences::InvoiceDatetimeFormat)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .drop_column(GandolaPreferences::InvoiceDateFormat)
                    .to_owned(),
            )
            .await
    }
}
