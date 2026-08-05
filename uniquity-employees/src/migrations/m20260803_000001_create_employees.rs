use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Employees {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    UserId,
}

#[derive(DeriveIden)]
enum PointsTransactions {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Points,
    FromUserId,
    ToEmployeeId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Employees::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Employees::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Employees::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Employees::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Employees::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Employees::UserId).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_employees_user_id")
                            .from(Employees::Table, Employees::UserId)
                            .to(Users::Table, Users::Id)
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
                    .name("idx_employees_deleted_at")
                    .table(Employees::Table)
                    .col(Employees::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_employees_user_id")
                    .table(Employees::Table)
                    .col(Employees::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PointsTransactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PointsTransactions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PointsTransactions::CreatedAt).timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(PointsTransactions::UpdatedAt).timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(PointsTransactions::DeletedAt).timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(PointsTransactions::Points)
                            .decimal_len(19, 2)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PointsTransactions::FromUserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PointsTransactions::ToEmployeeId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_points_transactions_from_user_id")
                            .from(PointsTransactions::Table, PointsTransactions::FromUserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_points_transactions_to_employee_id")
                            .from(PointsTransactions::Table, PointsTransactions::ToEmployeeId)
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
                    .name("idx_points_transactions_deleted_at")
                    .table(PointsTransactions::Table)
                    .col(PointsTransactions::DeletedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PointsTransactions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Employees::Table).to_owned())
            .await
    }
}
