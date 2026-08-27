//! Rune sandbox bindings for site purchase-order invoicing.

use std::sync::Arc;

use lariv_rs::rune_env::{NativeBinding, RuneEnvCapability, RuneEnvCtx, RuneEnvRegistrar};
use serde::Deserialize;
use serde_json::json;

/// Registers site invoicing helpers onto the assistant Rune environment.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl RuneEnvRegistrar for Hook {
    fn register_rune_env(self, rune_env: &mut RuneEnvCapability) {
        rune_env.register_contextual(
            "find_site",
            "find_site(#{ site_id?: int|string, name?: string, site_name?: string }) -> #{ id, site_id?, name, customer_id, address?, status } — int site_id is the primary key; string site_id / name / site_name match name or string site_id",
            |_ctx| NativeBinding::Function(Arc::new(find_site)),
        );
        rune_env.register_contextual(
            "list_site_purchase_orders",
            "list_site_purchase_orders(#{ site_id?: int|string, name?: string, site_name?: string }) -> #{ site: SiteSummary, purchase_orders: [#{ id, number, date, customer_id, already_invoiced, existing_invoice_id?, lines: [...] }] }",
            |_ctx| NativeBinding::Function(Arc::new(list_site_purchase_orders)),
        );
        rune_env.register_contextual(
            "create_invoices_for_site",
            "create_invoices_for_site(#{ site_id?: int|string, site_name?: string, name?: string, timezone?: string, dry_run?: bool }) -> #{ site_id, site_name, dry_run, created: [InvoiceAction], skipped: [InvoiceAction], errors: [InvoiceAction] }",
            |_ctx| NativeBinding::Function(Arc::new(create_invoices_for_site)),
        );
        rune_env.register_contextual(
            "link_site_invoice",
            "link_site_invoice(#{ site_id: int, invoice_id?: int, draft_invoice_id?: int }) -> #{ site_id, invoice_id, linked: true }",
            |_ctx| NativeBinding::Function(Arc::new(link_site_invoice)),
        );
    }
}

fn find_site(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed: SiteLookupArgs = parse_object_args(args, "find_site")?;
    let db = ctx.db.clone();
    let site = block_on_async(async move {
        crate::invoice_site_pos::find_site(&db, parsed.lookup_pk(), parsed.lookup_text()).await
    })?;
    lariv_rs::rune_env::json_to_rune(json!(crate::invoice_site_pos::site_summary(&site)))
}

fn list_site_purchase_orders(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed: SiteLookupArgs = parse_object_args(args, "list_site_purchase_orders")?;
    let db = ctx.db.clone();
    let (site, purchase_orders) = block_on_async(async move {
        let site =
            crate::invoice_site_pos::find_site(&db, parsed.lookup_pk(), parsed.lookup_text()).await?;
        let (_, pos) = crate::invoice_site_pos::list_site_purchase_orders(&db, site.id).await?;
        Ok::<_, String>((site, pos))
    })?;
    lariv_rs::rune_env::json_to_rune(json!({
        "site": crate::invoice_site_pos::site_summary(&site),
        "purchase_orders": purchase_orders,
    }))
}

fn create_invoices_for_site(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let parsed: CreateInvoicesArgs = parse_object_args(args, "create_invoices_for_site")?;
    let timezone = parsed.timezone.clone().unwrap_or_else(|| "UTC".to_string());
    let site_text = parsed.lookup_text().map(str::to_string);
    let site_pk = parsed.lookup_pk();
    let dry_run = parsed.dry_run.unwrap_or(false);
    let db = ctx.db.clone();
    let result = block_on_async(async move {
        crate::invoice_site_pos::create_invoices_for_site(
            &db,
            site_pk,
            site_text.as_deref(),
            &timezone,
            dry_run,
        )
        .await
    })?;
    lariv_rs::rune_env::json_to_rune(json!(result))
}

fn link_site_invoice(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let parsed: LinkSiteInvoiceArgs = parse_object_args(args, "link_site_invoice")?;
    let site_id = parsed.site_id;
    let invoice_id = parsed.invoice_id()?;
    let db = ctx.db.clone();
    block_on_async(async move {
        crate::scope::link_site_invoice(&db, site_id, invoice_id).await
    })?;
    lariv_rs::rune_env::json_to_rune(json!({
        "site_id": site_id,
        "invoice_id": invoice_id,
        "linked": true,
    }))
}

fn block_on_async<T, F>(fut: F) -> T
where
    T: Send,
    F: std::future::Future<Output = T> + Send,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(fut))
        }
        Ok(_) | Err(_) => std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("rune native runtime")
                        .block_on(fut)
                })
                .join()
                .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
        }),
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FlexibleSiteId {
    Id(i64),
    Code(String),
}

#[derive(Debug, Deserialize)]
struct SiteLookupArgs {
    #[serde(default)]
    site_id: Option<FlexibleSiteId>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    site_name: Option<String>,
}

impl SiteLookupArgs {
    fn lookup_pk(&self) -> Option<i64> {
        match &self.site_id {
            Some(FlexibleSiteId::Id(id)) => Some(*id),
            _ => None,
        }
    }

    fn lookup_text(&self) -> Option<&str> {
        match &self.site_id {
            Some(FlexibleSiteId::Code(code)) => {
                let trimmed = code.trim();
                if trimmed.is_empty() {
                    self.name.as_deref().or(self.site_name.as_deref())
                } else {
                    Some(trimmed)
                }
            }
            _ => self.name.as_deref().or(self.site_name.as_deref()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateInvoicesArgs {
    #[serde(default)]
    site_id: Option<FlexibleSiteId>,
    #[serde(default)]
    site_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    dry_run: Option<bool>,
}

impl CreateInvoicesArgs {
    fn lookup_pk(&self) -> Option<i64> {
        match &self.site_id {
            Some(FlexibleSiteId::Id(id)) => Some(*id),
            _ => None,
        }
    }

    fn lookup_text(&self) -> Option<&str> {
        match &self.site_id {
            Some(FlexibleSiteId::Code(code)) => {
                let trimmed = code.trim();
                if trimmed.is_empty() {
                    self.site_name.as_deref().or(self.name.as_deref())
                } else {
                    Some(trimmed)
                }
            }
            _ => self.site_name.as_deref().or(self.name.as_deref()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct LinkSiteInvoiceArgs {
    site_id: i64,
    #[serde(default)]
    invoice_id: Option<i64>,
    #[serde(default)]
    draft_invoice_id: Option<i64>,
}

impl LinkSiteInvoiceArgs {
    fn invoice_id(&self) -> Result<i64, String> {
        self.invoice_id
            .or(self.draft_invoice_id)
            .ok_or_else(|| "link_site_invoice requires invoice_id".into())
    }
}

fn parse_object_args<T: for<'de> Deserialize<'de>>(
    args: &[rune::Value],
    fn_name: &str,
) -> Result<T, String> {
    let value = args
        .first()
        .ok_or_else(|| format!("{fn_name} requires an object argument"))?;
    serde_json::from_value(lariv_rs::rune_env::rune_to_json(value)?)
        .map_err(|e| format!("invalid {fn_name} arguments: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use lariv_rs::plugins::filesystem::storage::{DynFilestore, UnimplementedFilestore};
    use sea_orm::DatabaseConnection;

    fn test_env_ctx<'a>(
        db: &'a DatabaseConnection,
        store: &'a Arc<DynFilestore>,
    ) -> RuneEnvCtx<'a> {
        RuneEnvCtx {
            db,
            store: Arc::clone(store),
        }
    }

    fn registered_env() -> RuneEnvCapability {
        let mut cap = RuneEnvCapability::new();
        Hook.register_rune_env(&mut cap);
        cap
    }

    #[test]
    fn registers_site_invoice_bindings() {
        let names = registered_env().all_names();
        for expected in [
            "find_site",
            "list_site_purchase_orders",
            "create_invoices_for_site",
            "link_site_invoice",
        ] {
            assert!(
                names.iter().any(|name| name == expected),
                "expected {expected} in {names:?}"
            );
        }
    }

    fn assert_create_invoices_rejects_missing_site() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "create_invoices_for_site")
            .map(|(_, f)| f)
            .expect("create_invoices_for_site");
        let arg = rune::to_value(HashMap::<String, rune::Value>::new()).expect("empty object");
        let err = f(&env_ctx, &[arg]).expect_err("missing site should fail");
        assert!(
            err.contains("site_id") || err.contains("site_name"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn create_invoices_for_site_rejects_missing_site_on_current_thread() {
        assert_create_invoices_rejects_missing_site();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn create_invoices_for_site_rejects_missing_site() {
        assert_create_invoices_rejects_missing_site();
    }

    #[test]
    fn find_site_rejects_missing_argument() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "find_site")
            .map(|(_, f)| f)
            .expect("find_site");
        let err = f(&env_ctx, &[]).expect_err("missing argument should fail");
        assert!(
            err.contains("find_site requires an object argument"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn link_site_invoice_rejects_missing_invoice_id() {
        let cap = registered_env();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let resolved = cap.resolve(&env_ctx);
        let f = resolved
            .functions
            .iter()
            .find(|(name, _)| name == "link_site_invoice")
            .map(|(_, f)| f)
            .expect("link_site_invoice");
        let mut args = HashMap::<String, rune::Value>::new();
        args.insert("site_id".into(), rune::to_value(12i64).expect("site_id"));
        let arg = rune::to_value(args).expect("object");
        let err = f(&env_ctx, &[arg]).expect_err("missing invoice_id should fail");
        assert!(
            err.contains("invoice_id"),
            "unexpected error: {err}"
        );
    }
}
