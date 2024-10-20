use tokio::sync::Mutex;

use sea_orm::DatabaseConnection;

use crate::utils::config::Config;

#[derive(Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
    pub scanning_complete: Mutex<bool>,
    pub scanning_progress: Mutex<f64>,
}
