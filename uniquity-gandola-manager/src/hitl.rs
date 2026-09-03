//! HITL-gated Rune bindings for purchase-order and site deletes (require human approval).

use std::sync::Arc;

use lariv_rs::plugins::llm_assistant::hitl::{HitlCapability, HitlRegistrar};
use lariv_rs::rune_env::{NativeBinding, RuneEnvCtx};
use serde::Deserialize;
use serde_json::json;

/// Registers HITL delete helpers onto the assistant HITL capability.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl HitlRegistrar for Hook {
    fn register_hitl(self, hitl: &mut HitlCapability) {
        hitl.register(
            "delete_purchase_order",
            "delete_purchase_order(#{ id: int }) -> #{ id, deleted: true }  // requires human approval",
            |_ctx| NativeBinding::Function(Arc::new(delete_purchase_order)),
        );
        hitl.register(
            "delete_site",
            "delete_site(#{ id: int }) -> #{ id, deleted: true }  // requires human approval; fails if purchase orders still reference the site",
            |_ctx| NativeBinding::Function(Arc::new(delete_site)),
        );
    }
}

fn delete_purchase_order(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    let id = parse_id_args(args, "delete_purchase_order")?;
    let db = ctx.db.clone();
    crate::rune_env::block_on_async(async move {
        crate::po_persist::delete_purchase_order(&db, id).await
    })?;
    lariv_rs::rune_env::json_to_rune(json!({ "id": id, "deleted": true }))
}

fn delete_site(ctx: &RuneEnvCtx<'_>, args: &[rune::Value]) -> Result<rune::Value, String> {
    let id = parse_id_args(args, "delete_site")?;
    let db = ctx.db.clone();
    crate::rune_env::block_on_async(
        async move { crate::site_persist::delete_site(&db, id).await },
    )?;
    lariv_rs::rune_env::json_to_rune(json!({ "id": id, "deleted": true }))
}

fn parse_id_args(args: &[rune::Value], fn_name: &str) -> Result<i64, String> {
    let parsed: DeleteArgs = crate::rune_env::parse_object_args(args, fn_name)?;
    if parsed.id <= 0 {
        return Err(format!("{fn_name} requires a positive id"));
    }
    Ok(parsed.id)
}

#[derive(Debug, Deserialize)]
struct DeleteArgs {
    id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use lariv_rs::plugins::filesystem::storage::{DynFilestore, UnimplementedFilestore};
    use sea_orm::DatabaseConnection;

    fn test_env_ctx<'a>(
        db: &'a DatabaseConnection,
        store: &'a Arc<DynFilestore>,
    ) -> RuneEnvCtx<'a> {
        RuneEnvCtx {
            db,
            store: Arc::clone(store),
            session_id: None,
        }
    }

    fn registered_hitl() -> HitlCapability {
        let mut cap = HitlCapability::new();
        Hook.register_hitl(&mut cap);
        cap
    }

    fn call<'a>(
        hitl: &HitlCapability,
        name: &str,
        env_ctx: &RuneEnvCtx<'a>,
        args: &[rune::Value],
    ) -> Result<rune::Value, String> {
        let resolved = hitl.resolve(env_ctx);
        let f = resolved
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f)
            .unwrap_or_else(|| panic!("expected {name}"));
        f(env_ctx, args)
    }

    #[test]
    fn registers_delete_bindings() {
        let names = registered_hitl().all_names();
        for expected in ["delete_purchase_order", "delete_site"] {
            assert!(
                names.iter().any(|name| name == expected),
                "expected {expected} in {names:?}"
            );
        }
    }

    #[test]
    fn delete_purchase_order_rejects_missing_id() {
        let hitl = registered_hitl();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let arg = rune::to_value(std::collections::HashMap::<String, rune::Value>::new())
            .expect("empty object");
        let err = call(&hitl, "delete_purchase_order", &env_ctx, &[arg])
            .expect_err("missing id should fail");
        assert!(err.contains("id"), "unexpected error: {err}");
    }

    #[test]
    fn delete_purchase_order_rejects_non_positive_id() {
        let hitl = registered_hitl();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let mut args = std::collections::HashMap::<String, rune::Value>::new();
        args.insert("id".into(), rune::to_value(0i64).expect("id"));
        let arg = rune::to_value(args).expect("object");
        let err = call(&hitl, "delete_purchase_order", &env_ctx, &[arg])
            .expect_err("non-positive id should fail");
        assert!(err.contains("positive"), "unexpected error: {err}");
    }

    #[test]
    fn delete_site_rejects_missing_argument() {
        let hitl = registered_hitl();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let err =
            call(&hitl, "delete_site", &env_ctx, &[]).expect_err("missing argument should fail");
        assert!(
            err.contains("delete_site requires an object argument"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn delete_site_rejects_non_positive_id() {
        let hitl = registered_hitl();
        let db = DatabaseConnection::default();
        let store: Arc<DynFilestore> = Arc::new(UnimplementedFilestore);
        let env_ctx = test_env_ctx(&db, &store);
        let mut args = std::collections::HashMap::<String, rune::Value>::new();
        args.insert("id".into(), rune::to_value(0i64).expect("id"));
        let arg = rune::to_value(args).expect("object");
        let err =
            call(&hitl, "delete_site", &env_ctx, &[arg]).expect_err("non-positive id should fail");
        assert!(err.contains("positive"), "unexpected error: {err}");
    }
}
