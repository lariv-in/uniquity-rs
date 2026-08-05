use sea_orm::DatabaseConnection;

use crate::config::VideoConfig;

#[derive(Clone)]
pub struct VideoState {
    pub db: DatabaseConnection,
    pub config: VideoConfig,
    pub http: reqwest::Client,
}

impl VideoState {
    pub fn new(db: DatabaseConnection, config: VideoConfig) -> Self {
        Self {
            db,
            config,
            http: reqwest::Client::new(),
        }
    }
}
