use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum GandolaPreferences {
    Table,
    PurchaseOrderFilesDirectoryId,
}

#[derive(DeriveIden)]
enum FilesystemNodes {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .add_column(
                        ColumnDef::new(GandolaPreferences::PurchaseOrderFilesDirectoryId)
                            .big_integer(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_gandola_preferences_po_files_directory_id")
                            .from_tbl(GandolaPreferences::Table)
                            .from_col(GandolaPreferences::PurchaseOrderFilesDirectoryId)
                            .to_tbl(FilesystemNodes::Table)
                            .to_col(FilesystemNodes::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
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
                    .drop_foreign_key(Alias::new("fk_gandola_preferences_po_files_directory_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GandolaPreferences::Table)
                    .drop_column(GandolaPreferences::PurchaseOrderFilesDirectoryId)
                    .to_owned(),
            )
            .await
    }
}
