use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PurchaseOrders {
    Table,
    SiteId,
}

#[derive(DeriveIden)]
enum Sites {
    Table,
    Id,
}

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
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .add_column(ColumnDef::new(PurchaseOrders::SiteId).big_integer())
                    .to_owned(),
            )
            .await?;

        execute(
            manager,
            r#"
            UPDATE purchase_orders po
            SET site_id = (
                SELECT s.id
                FROM sites s
                WHERE s.customer_id = po.customer_id
                ORDER BY s.id
                LIMIT 1
            )
            WHERE site_id IS NULL
            "#,
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .modify_column(
                        ColumnDef::new(PurchaseOrders::SiteId)
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_purchase_orders_site_id")
                            .from_tbl(PurchaseOrders::Table)
                            .from_col(PurchaseOrders::SiteId)
                            .to_tbl(Sites::Table)
                            .to_col(Sites::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_purchase_orders_site_id")
                    .table(PurchaseOrders::Table)
                    .col(PurchaseOrders::SiteId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_purchase_orders_site_id")
                    .table(PurchaseOrders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .drop_foreign_key(Alias::new("fk_purchase_orders_site_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .drop_column(PurchaseOrders::SiteId)
                    .to_owned(),
            )
            .await
    }
}
