use async_channel::Sender;
use rand_core::{OsRng, RngCore};
use snowflake::Snowflake;
use tokio::sync::oneshot;

use super::constants::{SECURE_ID_CHARSET, SECURE_ID_LENGTH, SECURE_ID_MASK};

#[derive(Debug)]
pub struct IDGenerator {
    snowflake_id_generator_request: Sender<oneshot::Sender<i64>>,
}

impl Default for IDGenerator {
    fn default() -> Self {
        let physical_cpu_count = num_cpus::get_physical().min(1024);

        // a shared mpmc channel between all the snowflake id generators
        let (snowflake_id_generator_request, request_receiver): (
            async_channel::Sender<oneshot::Sender<i64>>,
            async_channel::Receiver<oneshot::Sender<i64>>,
        ) = async_channel::bounded(physical_cpu_count);

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
                    if let Ok(oneshot_sender) = receiver.recv().await {
                        if oneshot_sender.send(sfgen.generate_id() as i64).is_err() {
                            tracing::error!("can't send snowflake id to oneshot channel");
                        }
                    };
                }
            });
        }

        Self {
            snowflake_id_generator_request,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateIDErr {
    #[error("can't send a request to generate snowflake id")]
    SendRequest,
    #[error("can't get snowflake id from oneshot channel")]
    GetID,
    #[error("generating new snowflake id should not take more than 1 second")]
    TimeOut,
}

impl IDGenerator {
    pub async fn snowflake(&self) -> Result<i64, GenerateIDErr> {
        let (tx, rx) = oneshot::channel();

        self.snowflake_id_generator_request
            .send(tx)
            .await
            .map_err(|_| GenerateIDErr::SendRequest)?;

        tokio::select! {
            new_id = rx => new_id.map_err(|_| GenerateIDErr::GetID),
            () = tokio::time::sleep(std::time::Duration::from_secs(1)) => Err(GenerateIDErr::TimeOut),
        }
    }

    pub fn secure(&self) -> String {
        let mut id = String::with_capacity(SECURE_ID_LENGTH);
        let mut bytes = vec![0u8; SECURE_ID_LENGTH];
        OsRng.fill_bytes(&mut bytes);
        for &byte in &bytes {
            id.push(SECURE_ID_CHARSET[(byte as usize) & SECURE_ID_MASK]);
        }
        id
    }
}
