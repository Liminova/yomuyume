use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_channel::{bounded, Receiver, Sender};
use rand_core::{OsRng, RngCore};
use snowflake::Snowflake;
use tokio::{
    sync::{oneshot, Mutex},
    time::sleep,
};

use super::{SECURE_ID_CHARSET, SECURE_ID_LENGTH, SECURE_ID_MASK};
use crate::utils::config::Config;

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Config,
    pub scanning_complete: Mutex<bool>,
    pub scanning_progress: Mutex<f64>,

    snowflake_id_generator_request: Sender<oneshot::Sender<i64>>,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, config: Config) -> Self {
        let physical_cpu_count = num_cpus::get_physical().min(1024);

        // a shared mpmc channel between all the snowflake id generators
        let (request_sender, request_receiver): (
            Sender<oneshot::Sender<i64>>,
            Receiver<oneshot::Sender<i64>>,
        ) = bounded(physical_cpu_count);

        // spin up the snowflake id generators
        for worker_id in 0..physical_cpu_count {
            let receiver = request_receiver.clone();
            tokio::spawn(async move {
                let mut sfgen = Snowflake::new(0, worker_id as u64, 0)
                    .with_datacenter_id_bits(0)
                    .with_worker_id_bits(10)
                    .build()
                    .expect("can't build snowflake generator");

                // waiting for requests
                loop {
                    match receiver.recv().await {
                        Ok(oneshot_sender) => {
                            if oneshot_sender.send(sfgen.generate_id() as i64).is_err() {
                                tracing::error!("can't send snowflake id to oneshot channel");
                            }
                        }
                        Err(_) => {
                            tracing::error!("snowflake generator channel closed");
                            continue;
                        }
                    };
                }
            });
        }

        Self {
            pool,
            config,
            scanning_complete: Mutex::new(false),
            scanning_progress: Mutex::new(0.0),

            snowflake_id_generator_request: request_sender,
        }
    }

    pub async fn generate_snowflake_id(&self) -> Result<i64> {
        let (tx, rx) = oneshot::channel();

        self.snowflake_id_generator_request
            .send(tx)
            .await
            .map_err(|e| anyhow!("can't request to generate snowflake id: {e:?}"))?;

        tokio::select! {
            new_id = rx => new_id.context("can't get snowflake id from oneshot channel"),
            _ = sleep(Duration::from_secs(1)) => Err(anyhow!("generating new snowflake id should not take more than 1 second"))
        }
    }

    pub fn generate_secure_id(&self) -> String {
        let mut id = String::with_capacity(SECURE_ID_LENGTH);
        let mut bytes = vec![0u8; SECURE_ID_LENGTH];
        OsRng.fill_bytes(&mut bytes);
        for &byte in &bytes {
            id.push(SECURE_ID_CHARSET[(byte as usize) & SECURE_ID_MASK]);
        }
        id
    }
}
