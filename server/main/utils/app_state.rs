use chrono::TimeDelta;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::utils::{
    cacher::{Cacher, CacherBuilder},
    config::Config,
    constants::{CACHER_SWEEP_INTERVAL_SECONDS, CACHER_TIME_TO_LIVE_HOURS},
    id_generator::IDGenerator,
};

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Config,
    pub scanning_complete: RwLock<bool>,
    pub scanning_progress: RwLock<f64>,
    pub id_generator: IDGenerator,
    pub cacher: Cacher,
}

impl AppState {
    pub async fn new() -> Arc<Self> {
        let config = Config::init();

        let pool = PgPoolOptions::new()
            .max_connections(100)
            .connect(&config.database_url)
            .await
            .expect("can't connect to database");

        let id_generator = IDGenerator::new(config.snowflake_thread);

        Arc::new(Self {
            pool,
            config,
            scanning_complete: RwLock::new(false),
            scanning_progress: RwLock::new(0.0),
            id_generator,
            cacher: CacherBuilder::default()
                .sweep_intervval(tokio::time::Duration::from_secs(
                    CACHER_SWEEP_INTERVAL_SECONDS,
                ))
                .time_to_live(TimeDelta::hours(CACHER_TIME_TO_LIVE_HOURS))
                .build(),
        })
    }
}
