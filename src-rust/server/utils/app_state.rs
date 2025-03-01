use dashmap::DashMap;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    structs::{
        ids::{SessionID, UserID},
        user_cache::UserCache,
    },
    utils::{config::Config, id_generator::IDGenerator, live_config::LiveConfig},
};

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Config,
    pub live_config: LiveConfig,
    pub scanning_complete: RwLock<bool>,
    pub scanning_progress: RwLock<f64>,
    pub id_generator: IDGenerator,
    pub user_cache: DashMap<UserID, UserCache>,
    pub session_cache: DashMap<SessionID, UserID>,
}

impl AppState {
    pub async fn new() -> Arc<Self> {
        let config = Config::init();

        let pool = PgPoolOptions::new()
            .max_connections(100)
            .connect(&config.database_url)
            .await
            .expect("can't connect to database");

        let live_config = LiveConfig::load(&pool)
            .await
            .expect("can't initialize live config");

        Arc::new(Self {
            pool,
            config,
            live_config,
            scanning_complete: RwLock::new(false),
            scanning_progress: RwLock::new(0.0),
            id_generator: IDGenerator::default(),
            user_cache: DashMap::new(),
            session_cache: DashMap::new(),
        })
    }
}
