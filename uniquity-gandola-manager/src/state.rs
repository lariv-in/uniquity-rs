use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct GandolaManagerState {
    pub db: DatabaseConnection,
}

impl GandolaManagerState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
