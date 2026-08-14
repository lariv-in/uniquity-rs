use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Gandolas {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Name,
}

#[derive(DeriveIden)]
enum Sites {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Name,
    Address,
    StartDate,
    EndDate,
    CustomerId,
    Status,
    PoRent,
    PoDti,
    PoTpi,
    PoExtn1,
    PoExtn2,
    PoExtn3,
}

#[derive(DeriveIden)]
enum PGandolaSites {
    Table,
    GandolaId,
    SiteId,
}

#[derive(DeriveIden)]
enum GandolaPreferences {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    GandolaProductId,
    TpiProductId,
    DtiProductId,
    PaymentTermLinesJson,
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Id,
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
            .create_table(
                Table::create()
                    .table(Gandolas::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Gandolas::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Gandolas::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Gandolas::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Gandolas::Name).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sites::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sites::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sites::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Sites::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Sites::Name).text().not_null())
                    .col(ColumnDef::new(Sites::Address).text())
                    .col(ColumnDef::new(Sites::StartDate).date())
                    .col(ColumnDef::new(Sites::EndDate).date())
                    .col(ColumnDef::new(Sites::CustomerId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Sites::Status)
                            .string_len(32)
                            .not_null()
                            .default("started"),
                    )
                    .col(ColumnDef::new(Sites::PoRent).text())
                    .col(ColumnDef::new(Sites::PoDti).text())
                    .col(ColumnDef::new(Sites::PoTpi).text())
                    .col(ColumnDef::new(Sites::PoExtn1).text())
                    .col(ColumnDef::new(Sites::PoExtn2).text())
                    .col(ColumnDef::new(Sites::PoExtn3).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sites_customer_id")
                            .from(Sites::Table, Sites::CustomerId)
                            .to(Customers::Table, Customers::Id)
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
                    .name("idx_sites_customer_id")
                    .table(Sites::Table)
                    .col(Sites::CustomerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PGandolaSites::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PGandolaSites::GandolaId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PGandolaSites::SiteId)
                            .big_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(PGandolaSites::GandolaId)
                            .col(PGandolaSites::SiteId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_p_gandola_sites_gandola_id")
                            .from(PGandolaSites::Table, PGandolaSites::GandolaId)
                            .to(Gandolas::Table, Gandolas::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_p_gandola_sites_site_id")
                            .from(PGandolaSites::Table, PGandolaSites::SiteId)
                            .to(Sites::Table, Sites::Id)
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
                    .name("idx_p_gandola_sites_site_id")
                    .table(PGandolaSites::Table)
                    .col(PGandolaSites::SiteId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GandolaPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GandolaPreferences::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GandolaPreferences::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(GandolaPreferences::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(GandolaPreferences::GandolaProductId).big_integer())
                    .col(ColumnDef::new(GandolaPreferences::TpiProductId).big_integer())
                    .col(ColumnDef::new(GandolaPreferences::DtiProductId).big_integer())
                    .col(ColumnDef::new(GandolaPreferences::PaymentTermLinesJson).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_gandola_preferences_gandola_product_id")
                            .from(
                                GandolaPreferences::Table,
                                GandolaPreferences::GandolaProductId,
                            )
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_gandola_preferences_tpi_product_id")
                            .from(GandolaPreferences::Table, GandolaPreferences::TpiProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_gandola_preferences_dti_product_id")
                            .from(GandolaPreferences::Table, GandolaPreferences::DtiProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GandolaPreferences::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PGandolaSites::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sites::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Gandolas::Table).to_owned())
            .await
    }
}
