use argon2::password_hash::rand_core::{OsRng, RngCore};
use async_channel::Sender;
use snowflake::Snowflake;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct IDGenerator {
    snowflake_id_generator_request: Sender<oneshot::Sender<i64>>,
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
    pub fn new(snowflake_id_thread_count: usize) -> Self {
        // a shared mpmc channel between all the snowflake id generators
        let (snowflake_id_generator_request, request_receiver): (
            async_channel::Sender<oneshot::Sender<i64>>,
            async_channel::Receiver<oneshot::Sender<i64>>,
        ) = async_channel::bounded(snowflake_id_thread_count);

        // spin up the snowflake id generators
        for worker_id in 0..snowflake_id_thread_count {
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

    const SECURE_ID_LENGTH: usize = 32;
    const SECURE_ID_CHARSET: &[char; 64] = &[
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
        'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1',
        '2', '3', '4', '5', '6', '7', '8', '9', '-', '_',
    ];
    const SECURE_ID_MASK: usize = 63; // SECURE_ID_CHARSET.len().next_power_of_two() - 1

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

    pub fn secure(&self) -> Result<String, argon2::password_hash::rand_core::Error> {
        let mut id = String::with_capacity(Self::SECURE_ID_LENGTH);
        let mut bytes = vec![0u8; Self::SECURE_ID_LENGTH];
        OsRng.try_fill_bytes(&mut bytes)?;
        for &byte in &bytes {
            id.push(Self::SECURE_ID_CHARSET[(byte as usize) & Self::SECURE_ID_MASK]);
        }
        Ok(id)
    }
}
