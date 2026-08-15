//! Move purchase-order payment terms off invoice draft tables.

use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum PurchaseOrders {
    Table,
    DraftPaymentTermId,
    PaymentTermId,
}

#[derive(DeriveIden)]
enum PurchaseOrderPaymentTerms {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum PurchaseOrderPaymentTermLines {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    PurchaseOrderPaymentTermId,
    LineOrder,
    DateKind,
    DueDatetime,
    DueDuration,
    AmountKind,
    Amount,
    AmountPercentage,
}

const CREATE_DELETE_FN: &str = r#"
CREATE OR REPLACE FUNCTION delete_purchase_order_payment_term_for_deleted_po()
RETURNS trigger AS $$
BEGIN
  IF OLD.payment_term_id IS NOT NULL THEN
    DELETE FROM purchase_order_payment_terms WHERE id = OLD.payment_term_id;
  END IF;
  RETURN OLD;
END;
$$ LANGUAGE plpgsql
"#;

const CREATE_DELETE_TRIGGER: &str = r#"
CREATE TRIGGER trg_purchase_orders_delete_payment_term
AFTER DELETE ON purchase_orders
FOR EACH ROW EXECUTE PROCEDURE delete_purchase_order_payment_term_for_deleted_po()
"#;

async fn execute(manager: &SchemaManager<'_>, sql: &str) -> Result<(), DbErr> {
    manager
        .get_connection()
        .execute_unprepared(sql)
        .await
        .map(|_| ())
}

async fn copy_terms(manager: &SchemaManager<'_>) -> Result<Vec<i64>, DbErr> {
    let conn = manager.get_connection();
    let rows = conn
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT id, draft_payment_term_id FROM purchase_orders \
             WHERE draft_payment_term_id IS NOT NULL"
                .to_string(),
        ))
        .await?;

    let mut old_term_ids = Vec::new();
    for row in rows {
        let po_id: i64 = row.try_get("", "id")?;
        let old_term_id: i64 = row.try_get("", "draft_payment_term_id")?;
        old_term_ids.push(old_term_id);

        let new_term = conn
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "INSERT INTO purchase_order_payment_terms (created_at, updated_at) \
                 SELECT created_at, updated_at FROM draft_payment_terms WHERE id = $1 \
                 RETURNING id",
                [old_term_id.into()],
            ))
            .await?
            .ok_or_else(|| {
                DbErr::Custom(format!(
                    "draft payment term {old_term_id} missing for purchase order {po_id}"
                ))
            })?;
        let new_term_id: i64 = new_term.try_get("", "id")?;

        conn.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO purchase_order_payment_term_lines \
             (created_at, updated_at, purchase_order_payment_term_id, line_order, \
              date_kind, due_datetime, due_duration, amount_kind, amount, amount_percentage) \
             SELECT created_at, updated_at, $1, line_order, date_kind, due_datetime, \
                    due_duration, amount_kind, amount, amount_percentage \
             FROM draft_payment_term_lines WHERE draft_payment_term_id = $2",
            [new_term_id.into(), old_term_id.into()],
        ))
        .await?;

        conn.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE purchase_orders SET payment_term_id = $1 WHERE id = $2",
            [new_term_id.into(), po_id.into()],
        ))
        .await?;
    }
    Ok(old_term_ids)
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PurchaseOrderPaymentTerms::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTerms::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTerms::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTerms::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PurchaseOrderPaymentTermLines::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::PurchaseOrderPaymentTermId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::LineOrder)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::DateKind)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::DueDatetime)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(PurchaseOrderPaymentTermLines::DueDuration).big_integer())
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::AmountKind)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PurchaseOrderPaymentTermLines::Amount).decimal_len(19, 6))
                    .col(
                        ColumnDef::new(PurchaseOrderPaymentTermLines::AmountPercentage)
                            .decimal_len(19, 6),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_po_payment_term_lines_term_id")
                            .from(
                                PurchaseOrderPaymentTermLines::Table,
                                PurchaseOrderPaymentTermLines::PurchaseOrderPaymentTermId,
                            )
                            .to(
                                PurchaseOrderPaymentTerms::Table,
                                PurchaseOrderPaymentTerms::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .add_column(
                        ColumnDef::new(PurchaseOrders::PaymentTermId)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        let old_term_ids = copy_terms(manager).await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_purchase_orders_draft_payment_term_id")
                    .table(PurchaseOrders::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("uq_purchase_orders_draft_payment_term_id")
                    .table(PurchaseOrders::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .drop_column(PurchaseOrders::DraftPaymentTermId)
                    .to_owned(),
            )
            .await?;

        if !old_term_ids.is_empty() {
            let ids = old_term_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            execute(
                manager,
                &format!("DELETE FROM draft_payment_terms WHERE id IN ({ids})"),
            )
            .await?;
        }

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .unique()
                    .name("uq_purchase_orders_payment_term_id")
                    .table(PurchaseOrders::Table)
                    .col(PurchaseOrders::PaymentTermId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(PurchaseOrders::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk_purchase_orders_payment_term_id")
                            .from_tbl(PurchaseOrders::Table)
                            .from_col(PurchaseOrders::PaymentTermId)
                            .to_tbl(PurchaseOrderPaymentTerms::Table)
                            .to_col(PurchaseOrderPaymentTerms::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        execute(manager, CREATE_DELETE_FN).await?;
        execute(manager, CREATE_DELETE_TRIGGER).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "m00008 purchase order payment terms cannot be reversed".into(),
        ))
    }
}
