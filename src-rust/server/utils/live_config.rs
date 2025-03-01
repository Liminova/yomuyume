#![allow(clippy::struct_excessive_bools)]

use std::fmt::Display;

use chrono::Utc;
use tokio::sync::RwLock;

use crate::routes::errors::InternalErr;

/// Like [`Config`], but configurable from the frontend.
#[derive(Debug)]
pub struct LiveConfig {
    nomedia_support: RwLock<bool>,

    komga_oneshot_support: RwLock<bool>,
    komga_recycle_support: RwLock<bool>,

    rescan_enabled: RwLock<bool>,
    rescan_interval_in_minutes: RwLock<u32>,
}

#[derive(Debug)]
pub enum LiveConfigItem {
    NomediaSupport,
    KomgaOneshotSupport,
    KomgaRecycleSupport,
    RescanEnabled,
    RescanIntervalInMinutes,
}

use LiveConfigItem as Item;

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::NomediaSupport => write!(f, "nomedia_support"),
            Item::KomgaOneshotSupport => write!(f, "komga_oneshot_support"),
            Item::KomgaRecycleSupport => write!(f, "komga_recycle_support"),
            Item::RescanEnabled => write!(f, "rescan_enabled"),
            Item::RescanIntervalInMinutes => write!(f, "rescan_interval_in_minutes"),
        }
    }
}

impl PartialEq<String> for Item {
    fn eq(&self, other: &String) -> bool {
        &self.to_string() == other
    }
}

impl PartialEq<Item> for String {
    fn eq(&self, other: &Item) -> bool {
        self == &other.to_string()
    }
}

impl LiveConfig {
    pub async fn load(db: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
        let records = sqlx::query!(
            r#"SELECT id,
                value
            FROM live_config
            WHERE id IN (
                    SELECT id
                    FROM UNNEST($1::TEXT [])
                )"#,
            &[
                Item::NomediaSupport.to_string(),
                Item::KomgaOneshotSupport.to_string(),
                Item::KomgaRecycleSupport.to_string(),
                Item::RescanEnabled.to_string(),
                Item::RescanIntervalInMinutes.to_string(),
            ]
        )
        .fetch_all(db)
        .await?;

        macro_rules! get_bool {
            ($id:expr, $default:expr) => {
                RwLock::new(
                    records
                        .iter()
                        .find(|r| r.id == $id)
                        .map_or($default, |r| r.value == "true"),
                )
            };
        }

        let nomedia_support = get_bool!(Item::NomediaSupport, false);
        let komga_oneshot_support = get_bool!(Item::KomgaOneshotSupport, false);
        let komga_recycle_support = get_bool!(Item::KomgaRecycleSupport, false);
        let rescan_enabled = get_bool!(Item::RescanEnabled, true);
        let rescan_interval_in_minutes = RwLock::new(
            records
                .iter()
                .find(|r| r.id == Item::RescanIntervalInMinutes)
                .map_or(60, |r| r.value.parse::<u32>().unwrap_or(60)),
        );

        Ok(Self {
            nomedia_support,
            komga_oneshot_support,
            komga_recycle_support,
            rescan_enabled,
            rescan_interval_in_minutes,
        })
    }

    pub async fn set(
        &self,
        db: &sqlx::PgPool,
        config: &Item,
        value: &str,
    ) -> Result<(), InternalErr> {
        match (&config, value) {
            (Item::RescanIntervalInMinutes, value) => {
                if value.parse::<u32>().is_err() {
                    return Err(InternalErr::LiveConfig(
                        "value must be an integer".to_string(),
                    ));
                }
            }
            (_, value) if value != "true" && value != "false" => {
                return Err(InternalErr::LiveConfig(
                    "value must be 'true' or 'false'".to_string(),
                ));
            }
            _ => {}
        }

        sqlx::query!(
            "INSERT INTO live_config (id, value, last_updated_at)
            VALUES ($1, $2, $3) ON CONFLICT (id) DO
            UPDATE
            SET value = $2,
                last_updated_at = $3",
            config.to_string(),
            value,
            Utc::now()
        )
        .execute(db)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::DB(e)
        })?;

        match config {
            Item::NomediaSupport => *self.nomedia_support.write().await = value == "true",
            Item::KomgaOneshotSupport => {
                *self.komga_oneshot_support.write().await = value == "true";
            }
            Item::KomgaRecycleSupport => {
                *self.komga_recycle_support.write().await = value == "true";
            }
            Item::RescanEnabled => *self.rescan_enabled.write().await = value == "true",
            Item::RescanIntervalInMinutes => {
                *self.rescan_interval_in_minutes.write().await = value.parse().unwrap_or(60);
            }
        }

        Ok(())
    }

    pub async fn get_nomediasupport(&self) -> bool {
        *self.nomedia_support.read().await
    }

    pub async fn get_komga_oneshot_support(&self) -> bool {
        *self.komga_oneshot_support.read().await
    }

    pub async fn get_komga_recycle_support(&self) -> bool {
        *self.komga_recycle_support.read().await
    }

    pub async fn get_rescan_enabled(&self) -> bool {
        *self.rescan_enabled.read().await
    }

    pub async fn get_rescan_interval_in_minutes(&self) -> u32 {
        *self.rescan_interval_in_minutes.read().await
    }
}
