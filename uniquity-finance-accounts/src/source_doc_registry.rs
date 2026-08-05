use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

/// Loaded backing document for a resolved type/id pair.
pub trait SourceDocInstance: Send + Sync {
    fn source_doc_type(&self) -> &str;
    fn source_doc_id(&self) -> i64;
    fn detail_url(&self) -> String;
}

/// Describes how one document kind participates in linking and URLs.
#[async_trait]
pub trait SourceDocType: Send + Sync {
    fn source_doc_type(&self) -> &str;
    fn detail_url(&self, id: i64) -> String;
    async fn load_from_id(
        &self,
        db: &DatabaseConnection,
        id: i64,
    ) -> Result<Arc<dyn SourceDocInstance>>;
}

#[derive(Default)]
pub struct SourceDocRegistry {
    types: RwLock<HashMap<String, Arc<dyn SourceDocType>>>,
}

impl SourceDocRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, loader: Arc<dyn SourceDocType>) -> Result<()> {
        let key = loader.source_doc_type().to_string();
        let mut map = self
            .types
            .write()
            .map_err(|_| anyhow!("source doc registry lock poisoned"))?;
        if map.contains_key(&key) {
            return Err(anyhow!(
                "p_uniquity_finance_accounts: duplicate source doc type {key:?}"
            ));
        }
        map.insert(key, loader);
        Ok(())
    }

    pub fn get(&self, typ: &str) -> Option<Arc<dyn SourceDocType>> {
        self.types.read().ok()?.get(typ).cloned()
    }

    pub async fn resolve_instance(
        &self,
        db: &DatabaseConnection,
        typ: &str,
        id: i64,
    ) -> Result<Arc<dyn SourceDocInstance>> {
        if typ.is_empty() {
            return Err(anyhow!(
                "p_uniquity_finance_accounts: ResolveSourceDocInstance: empty type"
            ));
        }
        let loader = self
            .get(typ)
            .ok_or_else(|| {
                anyhow!("p_uniquity_finance_accounts: ResolveSourceDocInstance: unknown type {typ:?}")
            })?;
        let inst = loader.load_from_id(db, id).await?;
        if inst.source_doc_type() != typ {
            return Err(anyhow!(
                "p_uniquity_finance_accounts: ResolveSourceDocInstance: type mismatch: registry key {typ:?}, instance {:?}",
                inst.source_doc_type()
            ));
        }
        Ok(inst)
    }
}
