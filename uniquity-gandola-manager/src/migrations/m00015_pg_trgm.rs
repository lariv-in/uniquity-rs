use lariv_rs::db::trigram;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::create_gin_index(db, backend, "sites_name_trgm_idx", "sites", "name").await?;
        trigram::create_gin_index(db, backend, "sites_address_trgm_idx", "sites", "address")
            .await?;
        trigram::create_gin_index(
            db,
            backend,
            "purchase_orders_number_trgm_idx",
            "purchase_orders",
            "number",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = manager.get_database_backend();
        trigram::drop_gin_index(db, backend, "purchase_orders_number_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "sites_address_trgm_idx").await?;
        trigram::drop_gin_index(db, backend, "sites_name_trgm_idx").await?;
        Ok(())
    }
}
