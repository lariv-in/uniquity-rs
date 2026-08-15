//! Purchase orders point at draft payment terms instead of storing JSON lines.

use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

use lariv_rs::plugins::finance_invoices::logic::{
    default_payment_term_lines_json, parse_payment_term_lines_json, upsert_draft_payment_term_lines,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PurchaseOrders {
    Table,
    DraftPaymentTermId,
    PaymentTermLinesJson,
}

#[derive(DeriveIden)]
enum DraftPaymentTerms {
    Table,
    Id,
}

async fn backfill_terms(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    let rows = conn
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT id, payment_term_lines_json FROM purchase_orders".to_string(),
        ))
        .await?;

    for row in rows {
        let po_id: i64 = row.try_get("", "id")?;
        let raw: String = row
            .try_get("", "payment_term_lines_json")
            .unwrap_or_default();
        let lines = parse_payment_term_lines_json(&raw)
            .or_else(|_| parse_payment_term_lines_json(&default_payment_term_lines_json()))
            .map_err(DbErr::Custom)?;
        let term = upsert_draft_payment_term_lines(conn, None, &lines, "UTC")
            .await
            .map_err(DbErr::Custom)?;
        conn.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE purchase_orders SET draft_payment_term_id = $1 WHERE id = $2",
            [term.id.into(), po_id.into()],
        ))
        .await?;
    }
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .add_column(
                        ColumnDef::new(PurchaseOrders::DraftPaymentTermId)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        backfill_terms(manager).await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .unique()
                    .name("uq_purchase_orders_draft_payment_term_id")
                    .table(PurchaseOrders::Table)
                    .col(PurchaseOrders::DraftPaymentTermId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_purchase_orders_draft_payment_term_id")
                            .from_tbl(PurchaseOrders::Table)
                            .from_col(PurchaseOrders::DraftPaymentTermId)
                            .to_tbl(DraftPaymentTerms::Table)
                            .to_col(DraftPaymentTerms::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .drop_column(PurchaseOrders::PaymentTermLinesJson)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "m00007 purchase order payment term fk cannot be reversed".into(),
        ))
    }
}
