//! LLM tools for searching sites and purchase orders.

use crate::entities::purchase_order::{self, Entity as PurchaseOrderEntity};
use crate::entities::site::{self, Entity as SiteEntity};
use async_trait::async_trait;
use lariv_rs::db::trigram;
use lariv_rs::genai::FunctionDeclaration;
use lariv_rs::llm_tools::{LlmTool, LlmToolsCapability, ToolCtx, ToolsRegistrar};
use serde::Deserialize;
use serde_json::{Value, json};

/// Registers Gandola search tools onto the assistant.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl ToolsRegistrar for Hook {
    fn register_tools(self, tools: &mut LlmToolsCapability) {
        tools
            .register(SearchSitesTool)
            .register(SearchPurchaseOrdersTool);
    }
}

#[derive(Debug, Deserialize, Default)]
struct SearchArgs {
    #[serde(default)]
    query: String,
    #[serde(default)]
    limit: u64,
}

fn parse_query(args: Value) -> Result<(String, u64), String> {
    let parsed: SearchArgs = serde_json::from_value(args).unwrap_or_default();
    let query = parsed.query.trim().to_string();
    if query.is_empty() {
        return Err("query is required".into());
    }
    Ok((query, trigram::clamp_search_limit(parsed.limit)))
}

fn search_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "query": { "type": "string", "description": "Fuzzy search text (trigram / substring)" },
            "limit": { "type": "integer", "description": "Max results (default 20, max 50)" }
        },
        "required": ["query"]
    })
}

pub struct SearchSitesTool;

#[async_trait]
impl LlmTool for SearchSitesTool {
    fn name(&self) -> &str {
        "search_sites"
    }

    fn declaration(&self) -> FunctionDeclaration {
        FunctionDeclaration {
            name: "search_sites".into(),
            description: "Search sites by name or address using trigram fuzzy matching.".into(),
            parameters: Some(search_params()),
        }
    }

    async fn run(&self, ctx: &ToolCtx<'_>, args: Value) -> Result<Value, String> {
        let (query, limit) = parse_query(args)?;
        let rows = trigram::search::<SiteEntity, _>(
            ctx.db,
            &[site::Column::Name, site::Column::Address],
            &query,
            limit,
        )
        .await
        .map_err(|e| e.to_string())?;
        let results: Vec<Value> = rows
            .into_iter()
            .map(|s| {
                json!({
                    "id": s.id,
                    "name": s.name,
                    "address": s.address,
                    "customer_id": s.customer_id,
                    "status": s.status.as_str(),
                })
            })
            .collect();
        Ok(json!({ "results": results }))
    }
}

pub struct SearchPurchaseOrdersTool;

#[async_trait]
impl LlmTool for SearchPurchaseOrdersTool {
    fn name(&self) -> &str {
        "search_purchase_orders"
    }

    fn declaration(&self) -> FunctionDeclaration {
        FunctionDeclaration {
            name: "search_purchase_orders".into(),
            description: "Search purchase orders by number using trigram fuzzy matching.".into(),
            parameters: Some(search_params()),
        }
    }

    async fn run(&self, ctx: &ToolCtx<'_>, args: Value) -> Result<Value, String> {
        let (query, limit) = parse_query(args)?;
        let rows = trigram::search::<PurchaseOrderEntity, _>(
            ctx.db,
            &[purchase_order::Column::Number],
            &query,
            limit,
        )
        .await
        .map_err(|e| e.to_string())?;
        let results: Vec<Value> = rows
            .into_iter()
            .map(|po| {
                json!({
                    "id": po.id,
                    "number": po.number,
                    "date": po.date,
                    "customer_id": po.customer_id,
                    "site_id": po.site_id,
                })
            })
            .collect();
        Ok(json!({ "results": results }))
    }
}
