use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Sites {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum DraftInvoices {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum SiteInvoices {
    Table,
    SiteId,
    DraftInvoiceId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SiteInvoices::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SiteInvoices::SiteId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SiteInvoices::DraftInvoiceId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(SiteInvoices::SiteId)
                            .col(SiteInvoices::DraftInvoiceId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_site_invoices_site_id")
                            .from(SiteInvoices::Table, SiteInvoices::SiteId)
                            .to(Sites::Table, Sites::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_site_invoices_draft_invoice_id")
                            .from(SiteInvoices::Table, SiteInvoices::DraftInvoiceId)
                            .to(DraftInvoices::Table, DraftInvoices::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_site_invoices_draft_invoice_id")
                    .table(SiteInvoices::Table)
                    .col(SiteInvoices::DraftInvoiceId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SiteInvoices::Table).to_owned())
            .await
    }
}
