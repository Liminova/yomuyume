use tokio::sync::Mutex;

use crate::utils::{config::Config, id_generator::IDGenerator};

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Config,
    pub scanning_complete: Mutex<bool>,
    pub scanning_progress: Mutex<f64>,
    pub id_generator: IDGenerator,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, config: Config) -> Self {
        Self {
            pool,
            config,
            scanning_complete: Mutex::new(false),
            scanning_progress: Mutex::new(0.0),
            id_generator: IDGenerator::default(),
        }
    }
}
