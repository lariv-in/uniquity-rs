//! Replace the points-transaction trigger that checked `users.is_superuser`.
//! That column is gone; superuser is the `users.role` value `superuser`.

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
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                r#"
CREATE OR REPLACE FUNCTION uniquity_points_transaction_check_from_superuser() RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM users WHERE id = NEW.from_user_id AND role = 'superuser'
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

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
