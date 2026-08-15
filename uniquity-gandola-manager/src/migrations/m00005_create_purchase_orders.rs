use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PurchaseOrders {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Number,
    Date,
    CustomerId,
    FileId,
    PaymentTermLinesJson,
    BillingAddress,
    ShippingAddress,
    Cin,
}

#[derive(DeriveIden)]
enum PurchaseOrderLines {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    PurchaseOrderId,
    ProductId,
    Rate,
    Quantity,
}

#[derive(DeriveIden)]
enum PPurchaseOrderLineTaxes {
    Table,
    PurchaseOrderLineId,
    TaxId,
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum FilesystemNodes {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Taxes {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PurchaseOrders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PurchaseOrders::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PurchaseOrders::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(PurchaseOrders::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(PurchaseOrders::Number)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(PurchaseOrders::Date).date().not_null())
                    .col(
                        ColumnDef::new(PurchaseOrders::CustomerId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PurchaseOrders::FileId).big_integer())
                    .col(
                        ColumnDef::new(PurchaseOrders::PaymentTermLinesJson)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PurchaseOrders::BillingAddress).text())
                    .col(ColumnDef::new(PurchaseOrders::ShippingAddress).text())
                    .col(ColumnDef::new(PurchaseOrders::Cin).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_purchase_orders_customer_id")
                            .from(PurchaseOrders::Table, PurchaseOrders::CustomerId)
                            .to(Customers::Table, Customers::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_purchase_orders_file_id")
                            .from(PurchaseOrders::Table, PurchaseOrders::FileId)
                            .to(FilesystemNodes::Table, FilesystemNodes::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
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
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderLines::ProductId)
                            .big_integer()
                            .not_null(),
                    )
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
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_purchase_order_lines_product_id")
                            .from(PurchaseOrderLines::Table, PurchaseOrderLines::ProductId)
                            .to(Products::Table, Products::Id)
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
                    .name("idx_purchase_order_lines_purchase_order_id")
                    .table(PurchaseOrderLines::Table)
                    .col(PurchaseOrderLines::PurchaseOrderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PPurchaseOrderLineTaxes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PPurchaseOrderLineTaxes::PurchaseOrderLineId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PPurchaseOrderLineTaxes::TaxId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(PPurchaseOrderLineTaxes::PurchaseOrderLineId)
                            .col(PPurchaseOrderLineTaxes::TaxId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_p_po_line_taxes_line_id")
                            .from(
                                PPurchaseOrderLineTaxes::Table,
                                PPurchaseOrderLineTaxes::PurchaseOrderLineId,
                            )
                            .to(PurchaseOrderLines::Table, PurchaseOrderLines::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_p_po_line_taxes_tax_id")
                            .from(
                                PPurchaseOrderLineTaxes::Table,
                                PPurchaseOrderLineTaxes::TaxId,
                            )
                            .to(Taxes::Table, Taxes::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(PPurchaseOrderLineTaxes::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(PurchaseOrderLines::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PurchaseOrders::Table).to_owned())
            .await
    }
}
