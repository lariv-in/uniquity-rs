use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PurchaseOrders {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum PurchaseOrderLines {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    PurchaseOrderId,
    ProductId,
    ItemCode,
    Description,
    Unit,
    DeliveryDate,
    Rate,
    Quantity,
}

#[derive(DeriveIden)]
enum PPurchaseOrderLineTaxes {
    Table,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(PPurchaseOrderLineTaxes::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(PurchaseOrderLines::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        create_lines_table(manager, true).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PurchaseOrderLines::Table).to_owned())
            .await?;
        create_lines_table(manager, false).await
    }
}

async fn create_lines_table(manager: &SchemaManager<'_>, new_shape: bool) -> Result<(), DbErr> {
    let mut table = Table::create();
    table
        .table(PurchaseOrderLines::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(PurchaseOrderLines::Id)
                .big_integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(PurchaseOrderLines::CreatedAt).timestamp_with_time_zone())
        .col(ColumnDef::new(PurchaseOrderLines::UpdatedAt).timestamp_with_time_zone())
        .col(
            ColumnDef::new(PurchaseOrderLines::PurchaseOrderId)
                .big_integer()
                .not_null(),
        );
    if new_shape {
        table
            .col(
                ColumnDef::new(PurchaseOrderLines::ItemCode)
                    .text()
                    .not_null(),
            )
            .col(
                ColumnDef::new(PurchaseOrderLines::Description)
                    .text()
                    .not_null(),
            )
            .col(ColumnDef::new(PurchaseOrderLines::Unit).text().not_null())
            .col(
                ColumnDef::new(PurchaseOrderLines::DeliveryDate)
                    .date()
                    .not_null(),
            );
    } else {
        table.col(
            ColumnDef::new(PurchaseOrderLines::ProductId)
                .big_integer()
                .not_null(),
        );
    }
    table
        .col(
            ColumnDef::new(PurchaseOrderLines::Rate)
                .decimal_len(19, 6)
                .not_null(),
        )
        .col(
            ColumnDef::new(PurchaseOrderLines::Quantity)
                .decimal_len(19, 6)
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_purchase_order_lines_purchase_order_id")
                .from(
                    PurchaseOrderLines::Table,
                    PurchaseOrderLines::PurchaseOrderId,
                )
                .to(PurchaseOrders::Table, PurchaseOrders::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade),
        );
    if !new_shape {
        table.foreign_key(
            ForeignKey::create()
                .name("fk_purchase_order_lines_product_id")
                .from(PurchaseOrderLines::Table, PurchaseOrderLines::ProductId)
                .to(Products::Table, Products::Id)
                .on_delete(ForeignKeyAction::Restrict)
                .on_update(ForeignKeyAction::Cascade),
        );
    }
    manager.create_table(table.to_owned()).await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx_purchase_order_lines_purchase_order_id")
                .table(PurchaseOrderLines::Table)
                .col(PurchaseOrderLines::PurchaseOrderId)
                .to_owned(),
        )
        .await
}
