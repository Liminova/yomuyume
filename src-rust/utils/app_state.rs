use tokio::sync::Mutex;

use crate::utils::config::Config;

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Config,
    pub scanning_complete: Mutex<bool>,
    pub scanning_progress: Mutex<f64>,
}
