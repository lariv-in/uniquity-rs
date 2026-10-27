//! Site links survive posting: rows for posted invoices key off posted_invoice_id.
//! Must run while posted_invoices.draft_invoice_id still exists.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        // New columns first. Drop the composite PK before draft_invoice_id can
        // become nullable (Postgres refuses ALTER … DROP NOT NULL on a PK col).
        conn.execute_unprepared(
            "ALTER TABLE site_invoices ADD COLUMN IF NOT EXISTS id bigserial",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE site_invoices ADD COLUMN IF NOT EXISTS posted_invoice_id bigint",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE site_invoices DROP CONSTRAINT IF EXISTS site_invoices_pkey",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE site_invoices ALTER COLUMN draft_invoice_id DROP NOT NULL",
        )
        .await?;
        // Prefer remap while posted_invoices.draft_invoice_id still exists.
        // If finance m00044 already dropped that column (and deleted those
        // drafts), CASCADE will already have removed the matching site rows.
        conn.execute_unprepared(
            r#"DO $$
               BEGIN
                 IF EXISTS (
                   SELECT 1 FROM information_schema.columns
                   WHERE table_schema = current_schema()
                     AND table_name = 'posted_invoices'
                     AND column_name = 'draft_invoice_id'
                 ) THEN
                   UPDATE site_invoices AS si
                   SET posted_invoice_id = pi.id,
                       draft_invoice_id = NULL
                   FROM posted_invoices pi
                   WHERE pi.draft_invoice_id = si.draft_invoice_id;
                 END IF;
               END $$"#,
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE site_invoices DROP CONSTRAINT IF EXISTS fk_site_invoices_draft_invoice_id",
        )
        .await?;
        // Idempotent if a previous partial run already switched the PK to id.
        conn.execute_unprepared(
            r#"DO $$
               BEGIN
                 IF NOT EXISTS (
                   SELECT 1 FROM pg_constraint
                   WHERE conname = 'site_invoices_pkey'
                 ) THEN
                   ALTER TABLE site_invoices
                     ADD CONSTRAINT site_invoices_pkey PRIMARY KEY (id);
                 END IF;
               END $$"#,
        )
        .await?;
        conn.execute_unprepared(
            r#"DO $$
               BEGIN
                 IF NOT EXISTS (
                   SELECT 1 FROM pg_constraint
                   WHERE conname = 'fk_site_invoices_draft_invoice_id'
                 ) THEN
                   ALTER TABLE site_invoices
                     ADD CONSTRAINT fk_site_invoices_draft_invoice_id
                     FOREIGN KEY (draft_invoice_id) REFERENCES draft_invoices(id)
                     ON UPDATE CASCADE ON DELETE CASCADE;
                 END IF;
               END $$"#,
        )
        .await?;
        conn.execute_unprepared(
            r#"DO $$
               BEGIN
                 IF NOT EXISTS (
                   SELECT 1 FROM pg_constraint
                   WHERE conname = 'fk_site_invoices_posted_invoice_id'
                 ) THEN
                   ALTER TABLE site_invoices
                     ADD CONSTRAINT fk_site_invoices_posted_invoice_id
                     FOREIGN KEY (posted_invoice_id) REFERENCES posted_invoices(id)
                     ON UPDATE CASCADE ON DELETE CASCADE;
                 END IF;
               END $$"#,
        )
        .await?;
        conn.execute_unprepared(
            r#"ALTER TABLE site_invoices
               DROP CONSTRAINT IF EXISTS ck_site_invoices_draft_xor_posted"#,
        )
        .await?;
        conn.execute_unprepared(
            r#"ALTER TABLE site_invoices
               ADD CONSTRAINT ck_site_invoices_draft_xor_posted CHECK (
                 (draft_invoice_id IS NOT NULL AND posted_invoice_id IS NULL)
                 OR (draft_invoice_id IS NULL AND posted_invoice_id IS NOT NULL)
               )"#,
        )
        .await?;
        conn.execute_unprepared(
            r#"CREATE UNIQUE INDEX IF NOT EXISTS uix_site_invoices_site_draft
               ON site_invoices (site_id, draft_invoice_id)
               WHERE draft_invoice_id IS NOT NULL"#,
        )
        .await?;
        conn.execute_unprepared(
            r#"CREATE UNIQUE INDEX IF NOT EXISTS uix_site_invoices_site_posted
               ON site_invoices (site_id, posted_invoice_id)
               WHERE posted_invoice_id IS NOT NULL"#,
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_site_invoices_posted_invoice_id ON site_invoices (posted_invoice_id)",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        // Best-effort: cannot restore deleted draft links after finance m00044.
        conn.execute_unprepared(
            "ALTER TABLE site_invoices DROP CONSTRAINT IF EXISTS ck_site_invoices_draft_xor_posted",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE site_invoices DROP CONSTRAINT IF EXISTS fk_site_invoices_posted_invoice_id",
        )
        .await?;
        conn.execute_unprepared("DROP INDEX IF EXISTS uix_site_invoices_site_posted")
            .await?;
        conn.execute_unprepared("DROP INDEX IF EXISTS uix_site_invoices_site_draft")
            .await?;
        conn.execute_unprepared("DROP INDEX IF EXISTS idx_site_invoices_posted_invoice_id")
            .await?;
        conn.execute_unprepared(
            "ALTER TABLE site_invoices DROP COLUMN IF EXISTS posted_invoice_id",
        )
        .await?;
        Ok(())
    }
}
