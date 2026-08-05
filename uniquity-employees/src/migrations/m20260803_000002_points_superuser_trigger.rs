use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

use uniquity_common::schema::is_postgres;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !is_postgres(manager) {
            return Ok(());
        }

        let backend = manager.get_connection().get_database_backend();
        let conn = manager.get_connection();

        conn.execute(Statement::from_string(
            backend,
            r#"
CREATE OR REPLACE FUNCTION uniquity_points_transaction_check_from_superuser() RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM users WHERE id = NEW.from_user_id AND is_superuser IS TRUE
  ) THEN
    RAISE EXCEPTION 'from_user_id must reference a superuser';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql
"#
            .to_string(),
        ))
        .await?;

        conn.execute(Statement::from_string(
            backend,
            "DROP TRIGGER IF EXISTS uniquity_points_transaction_bi ON points_transactions".to_string(),
        ))
        .await?;

        conn.execute(Statement::from_string(
            backend,
            r#"
CREATE TRIGGER uniquity_points_transaction_bi
  BEFORE INSERT ON points_transactions
  FOR EACH ROW EXECUTE FUNCTION uniquity_points_transaction_check_from_superuser()
"#
            .to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !is_postgres(manager) {
            return Ok(());
        }

        let backend = manager.get_connection().get_database_backend();
        let conn = manager.get_connection();

        conn.execute(Statement::from_string(
            backend,
            "DROP TRIGGER IF EXISTS uniquity_points_transaction_bi ON points_transactions".to_string(),
        ))
        .await?;

        conn.execute(Statement::from_string(
            backend,
            "DROP FUNCTION IF EXISTS uniquity_points_transaction_check_from_superuser()".to_string(),
        ))
        .await?;

        Ok(())
    }
}
