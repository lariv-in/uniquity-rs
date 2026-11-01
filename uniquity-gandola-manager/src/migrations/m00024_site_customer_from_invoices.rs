//! Prefer invoice `customer_id` on sites (and their purchase orders).
//!
//! Site / PO `customer_id` values are less trustworthy than finance bill-to.
//! For each site whose linked invoices agree on a single customer, overwrite
//! `sites.customer_id` and matching `purchase_orders.customer_id`. Sites with
//! conflicting invoice customers fail the migration.

use lariv_core::db::migration_sql::exec_sql;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const UP: &[&str] = &[
    r#"DO $fix$
BEGIN
    IF EXISTS (
        SELECT 1
          FROM (
              SELECT si.site_id,
                     COUNT(DISTINCT COALESCE(d.customer_id, p.customer_id)) AS n
                FROM site_invoices AS si
                LEFT JOIN draft_invoices AS d ON d.id = si.draft_invoice_id
                LEFT JOIN posted_invoices AS p ON p.id = si.posted_invoice_id
               WHERE COALESCE(d.customer_id, p.customer_id) IS NOT NULL
               GROUP BY si.site_id
          ) AS per_site
         WHERE per_site.n > 1
    ) THEN
        RAISE EXCEPTION
            'site has linked invoices with conflicting customer_id; resolve before remapping';
    END IF;
END
$fix$"#,
    r#"UPDATE sites AS s
       SET customer_id = inv.customer_id
       FROM (
           SELECT si.site_id,
                  MIN(COALESCE(d.customer_id, p.customer_id)) AS customer_id
             FROM site_invoices AS si
             LEFT JOIN draft_invoices AS d ON d.id = si.draft_invoice_id
             LEFT JOIN posted_invoices AS p ON p.id = si.posted_invoice_id
            WHERE COALESCE(d.customer_id, p.customer_id) IS NOT NULL
            GROUP BY si.site_id
       ) AS inv
       WHERE s.id = inv.site_id
         AND s.customer_id IS DISTINCT FROM inv.customer_id"#,
    r#"UPDATE purchase_orders AS po
       SET customer_id = s.customer_id
       FROM sites AS s
       WHERE po.site_id = s.id
         AND po.customer_id IS DISTINCT FROM s.customer_id
         AND EXISTS (
             SELECT 1
               FROM site_invoices AS si
              WHERE si.site_id = s.id
         )"#,
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for sql in UP {
            exec_sql(manager, sql).await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "site customer remap from invoices is not reversible".into(),
        ))
    }
}
