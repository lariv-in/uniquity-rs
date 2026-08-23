use sea_orm::DatabaseConnection;

use crate::po_import_queue::PoImportQueue;

#[derive(Clone)]
pub struct GandolaManagerState {
    pub db: DatabaseConnection,
    pub po_imports: PoImportQueue,
}

impl GandolaManagerState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            po_imports: PoImportQueue::new(),
        }
    }
}
