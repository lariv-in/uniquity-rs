use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct EmployeesState {
    pub db: DatabaseConnection,
}

impl EmployeesState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
