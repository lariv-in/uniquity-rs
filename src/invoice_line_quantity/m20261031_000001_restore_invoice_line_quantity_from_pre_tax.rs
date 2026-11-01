//! Restore line quantities wiped to 1 by `m20261009_000001_invoice_line_quantity_one`.
//!
//! Infers quantity from `pre_tax_amount / rate` only when quantity is still 1 and that
//! ratio is a clean value greater than 1 (true qty-1 lines stay unchanged).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const INVOICE_LINE_TABLES: [&str; 3] = [
    "draft_invoice_lines",
    "posted_invoice_lines",
    "cancelled_invoice_lines",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        for table in INVOICE_LINE_TABLES {
            conn.execute_unprepared(&format!(
                r#"DO $$
BEGIN
    IF to_regclass('public.{table}') IS NOT NULL THEN
        UPDATE {table}
           SET quantity = ROUND(pre_tax_amount / rate, 6)
         WHERE rate <> 0
           AND quantity = 1
           AND ROUND(pre_tax_amount / rate, 6) > 1
           AND ABS(pre_tax_amount - rate * ROUND(pre_tax_amount / rate, 6)) < 0.000001;
    END IF;
END $$;"#
            ))
            .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
