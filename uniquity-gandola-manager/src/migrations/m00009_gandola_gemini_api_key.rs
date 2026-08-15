use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum GandolaPreferences {
    Table,
    GeminiApiKey,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .add_column(
                        ColumnDef::new(GandolaPreferences::GeminiApiKey)
                            .text()
                            .not_null()
                            .default(""),
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
                    .drop_column(GandolaPreferences::GeminiApiKey)
                    .to_owned(),
            )
            .await
    }
}
