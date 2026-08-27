use lariv_rs::db::trigram;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Sites {
    Table,
    SiteId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Sites::Table)
                    .add_column(ColumnDef::new(Sites::SiteId).text())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::create_gin_index(db, backend, "sites_site_id_trgm_idx", "sites", "site_id")
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::drop_gin_index(db, backend, "sites_site_id_trgm_idx").await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Sites::Table)
                    .drop_column(Sites::SiteId)
                    .to_owned(),
            )
            .await
    }
}
