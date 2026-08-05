use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::source_doc_registry::SourceDocRegistry;

#[derive(Clone)]
pub struct AccountsState {
    pub db: DatabaseConnection,
    pub source_doc_registry: Arc<SourceDocRegistry>,
}

impl AccountsState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            source_doc_registry: Arc::new(SourceDocRegistry::new()),
        }
    }
}
