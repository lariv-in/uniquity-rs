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
        execute(
            manager,
            "DELETE FROM published_videos WHERE deleted_at IS NOT NULL",
        )
        .await?;
        execute(
            manager,
            "DELETE FROM edited_videos WHERE deleted_at IS NOT NULL",
        )
        .await?;
        execute(
            manager,
            "DELETE FROM raw_footages WHERE deleted_at IS NOT NULL",
        )
        .await?;

        for (index, table) in [
            ("idx_published_videos_deleted_at", "published_videos"),
            ("idx_edited_videos_deleted_at", "edited_videos"),
            ("idx_raw_footages_deleted_at", "raw_footages"),
        ] {
            manager
                .drop_index(Index::drop().name(index).table(Alias::new(table)).to_owned())
                .await?;
        }

        for table in ["published_videos", "edited_videos", "raw_footages"] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new(table))
                        .drop_column(Alias::new("deleted_at"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in ["raw_footages", "edited_videos", "published_videos"] {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new(table))
                        .add_column(
                            ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        for (index, table) in [
            ("idx_raw_footages_deleted_at", "raw_footages"),
            ("idx_edited_videos_deleted_at", "edited_videos"),
            ("idx_published_videos_deleted_at", "published_videos"),
        ] {
            manager
                .create_index(
                    Index::create()
                        .if_not_exists()
                        .name(index)
                        .table(Alias::new(table))
                        .col(Alias::new("deleted_at"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
