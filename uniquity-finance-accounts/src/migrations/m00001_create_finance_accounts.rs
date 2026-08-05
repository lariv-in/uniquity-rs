use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Name,
    Code,
    IsGroup,
    BalanceType,
    ParentId,
}

#[derive(DeriveIden)]
enum BalanceType {
    #[sea_orm(iden = "balance_type")]
    Enum,
    #[sea_orm(iden = "Credit")]
    Credit,
    #[sea_orm(iden = "Debit")]
    Debit,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(BalanceType::Enum)
                    .values([BalanceType::Credit, BalanceType::Debit])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Accounts::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Accounts::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Accounts::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Accounts::DeletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Accounts::Name).text().not_null())
                    .col(ColumnDef::new(Accounts::Code).integer().not_null())
                    .col(
                        ColumnDef::new(Accounts::IsGroup)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Accounts::BalanceType)
                            .custom(BalanceType::Enum)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Accounts::ParentId).big_integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_accounts_parent_id")
                            .from(Accounts::Table, Accounts::ParentId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_accounts_deleted_at")
                    .table(Accounts::Table)
                    .col(Accounts::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_accounts_parent_id")
                    .table(Accounts::Table)
                    .col(Accounts::ParentId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(BalanceType::Enum).to_owned())
            .await
    }
}
