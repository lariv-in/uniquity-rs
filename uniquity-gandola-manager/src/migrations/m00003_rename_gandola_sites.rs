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
        for sql in [
            "ALTER TABLE p_gandola_sites RENAME TO gandola_sites",
            "ALTER INDEX IF EXISTS idx_p_gandola_sites_site_id RENAME TO idx_gandola_sites_site_id",
            "ALTER TABLE gandola_sites RENAME CONSTRAINT fk_p_gandola_sites_gandola_id TO fk_gandola_sites_gandola_id",
            "ALTER TABLE gandola_sites RENAME CONSTRAINT fk_p_gandola_sites_site_id TO fk_gandola_sites_site_id",
            "ALTER TABLE gandola_sites RENAME CONSTRAINT p_gandola_sites_pkey TO gandola_sites_pkey",
        ] {
            execute(manager, sql).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for sql in [
            "ALTER TABLE gandola_sites RENAME CONSTRAINT gandola_sites_pkey TO p_gandola_sites_pkey",
            "ALTER TABLE gandola_sites RENAME CONSTRAINT fk_gandola_sites_site_id TO fk_p_gandola_sites_site_id",
            "ALTER TABLE gandola_sites RENAME CONSTRAINT fk_gandola_sites_gandola_id TO fk_p_gandola_sites_gandola_id",
            "ALTER INDEX IF EXISTS idx_gandola_sites_site_id RENAME TO idx_p_gandola_sites_site_id",
            "ALTER TABLE gandola_sites RENAME TO p_gandola_sites",
        ] {
            execute(manager, sql).await?;
        }
        Ok(())
    }
}
