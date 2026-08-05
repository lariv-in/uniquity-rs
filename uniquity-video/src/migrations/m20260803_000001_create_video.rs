use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum RawFootages {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Title,
    AssignedToId,
}

#[derive(DeriveIden)]
enum RawFootageFiles {
    Table,
    RawFootageId,
    VNodeId,
}

#[derive(DeriveIden)]
enum EditedVideos {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    RawFootageId,
    EditedVNodeId,
}

#[derive(DeriveIden)]
enum PublishedVideos {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    EditedVideoId,
    YouTubeVideoId,
}

#[derive(DeriveIden)]
enum Employees {
    Table,
    Id,
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
            .create_table(
                Table::create()
                    .table(RawFootages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RawFootages::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RawFootages::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(RawFootages::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(RawFootages::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(RawFootages::Title).text().not_null())
                    .col(
                        ColumnDef::new(RawFootages::AssignedToId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_raw_footages_assigned_to_id")
                            .from(RawFootages::Table, RawFootages::AssignedToId)
                            .to(Employees::Table, Employees::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_raw_footages_deleted_at")
                    .table(RawFootages::Table)
                    .col(RawFootages::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_raw_footages_assigned_to_id")
                    .table(RawFootages::Table)
                    .col(RawFootages::AssignedToId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RawFootageFiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RawFootageFiles::RawFootageId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RawFootageFiles::VNodeId).big_integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(RawFootageFiles::RawFootageId)
                            .col(RawFootageFiles::VNodeId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_raw_footage_files_raw_footage_id")
                            .from(RawFootageFiles::Table, RawFootageFiles::RawFootageId)
                            .to(RawFootages::Table, RawFootages::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_raw_footage_files_v_node_id")
                            .from(RawFootageFiles::Table, RawFootageFiles::VNodeId)
                            .to(FilesystemNodes::Table, FilesystemNodes::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EditedVideos::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EditedVideos::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(EditedVideos::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(EditedVideos::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(EditedVideos::DeletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(EditedVideos::RawFootageId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EditedVideos::EditedVNodeId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_edited_videos_raw_footage_id")
                            .from(EditedVideos::Table, EditedVideos::RawFootageId)
                            .to(RawFootages::Table, RawFootages::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_edited_videos_edited_v_node_id")
                            .from(EditedVideos::Table, EditedVideos::EditedVNodeId)
                            .to(FilesystemNodes::Table, FilesystemNodes::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_edited_videos_deleted_at")
                    .table(EditedVideos::Table)
                    .col(EditedVideos::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PublishedVideos::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PublishedVideos::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PublishedVideos::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(PublishedVideos::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(PublishedVideos::DeletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(PublishedVideos::EditedVideoId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PublishedVideos::YouTubeVideoId)
                            .string_len(32)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_published_videos_edited_video_id")
                            .from(PublishedVideos::Table, PublishedVideos::EditedVideoId)
                            .to(EditedVideos::Table, EditedVideos::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_published_videos_deleted_at")
                    .table(PublishedVideos::Table)
                    .col(PublishedVideos::DeletedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PublishedVideos::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(EditedVideos::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RawFootageFiles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RawFootages::Table).to_owned())
            .await
    }
}
