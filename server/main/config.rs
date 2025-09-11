use std::path::PathBuf;

use lettre::Address;

use crate::utils::{absolute_path::AbsolutePath, constants};

#[derive(Debug, Clone)]
pub struct Smtp {
    pub host: String,
    pub username: String,
    pub password: String,
    pub from_email: Address,
    pub from_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OperateMode {
    Public,
    RequireRegister,
    // TOOD: not implemented
    InviteOnly,
}

impl From<&str> for OperateMode {
    fn from(s: &str) -> OperateMode {
        match s.trim().to_lowercase().as_str() {
            "public" => OperateMode::Public,
            "require_register" => OperateMode::RequireRegister,
            "invite_only" => OperateMode::InviteOnly,
            _ => {
                tracing::warn!("unknown OPERATE_MODE: {}, using public", s);
                OperateMode::Public
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_address: String,

    pub library_path: AbsolutePath,
    pub database_url: String,
    pub tantivy_dir: AbsolutePath,
    pub tantivy_memory: usize,

    pub feature_nomedia: bool,
    pub feature_komga_oneshot: bool,
    pub feature_komga_recycle: bool,

    pub rescan_interval: tokio::time::Duration,
    pub operate_mode: OperateMode,

    pub smtp: Option<Smtp>,
}

macro_rules! must {
    ($env:expr) => {
        std::env::var($env).expect(concat!($env, " must be set"))
    };
}

macro_rules! optional {
    ($env:expr) => {
        std::env::var($env)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    ($env:expr, $default:expr) => {
        std::env::var($env)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| $default.to_string())
    };

    (num: $env:expr, $default:expr) => {
        std::env::var($env)
            .unwrap_or_else(|_| $default.to_string())
            .parse()
            .map_err(|e| {
                tracing::warn!(
                    "can't parse {} as a number: {}, using {}",
                    $env,
                    e,
                    $default
                )
            })
            .unwrap_or_else(|_| $default)
    };

    (bool: $env:expr, $default:expr) => {
        std::env::var($env)
            .unwrap_or_else(|_| $default.to_string())
            .parse()
            .map_err(|e| {
                tracing::warn!(
                    "can't parse {} as a boolean: {}, using {}",
                    $env,
                    e,
                    $default
                )
            })
            .unwrap_or_else(|_| $default)
    };
}

impl Config {
    pub fn init() -> Config {
        let library_path = AbsolutePath::from(&PathBuf::from(must!("LIBRARY_PATH")), None)
            .expect("can't convert LIBRARY_PATH to absolute");

        assert!(
            library_path.as_ref().is_dir(),
            "LIBRARY_PATH is not point to a valid directory"
        );

        Config {
            listen_address: optional!("LISTEN_ADDRESS", "0.0.0.0:3000"),

            library_path,
            database_url: optional!("DATABASE_URL", "sqlite://yomuyume.db?mode=rwc"),
            tantivy_dir: AbsolutePath::from(
                &PathBuf::from(optional!("TANTIVY_DIR", "/tmp/ymym-tantivy/")),
                None,
            )
            .expect("can't convert TANTIVY_DIR to absolute"),
            tantivy_memory: optional!(num: "TANTIVY_MEMORY_MB", 50) * 1_000_000,

            feature_nomedia: optional!(bool: "FEATURE_NOMEDIA", false),
            feature_komga_oneshot: optional!(bool: "FEATURE_KOMGA_ONESHOT", false),
            feature_komga_recycle: optional!(bool: "FEATURE_KOMGA_RECYCLE", false),

            rescan_interval: tokio::time::Duration::from_secs(
                optional!(num: "RESCAN_INTERVAL_SECS", 6 * 60 * 60),
            ),
            operate_mode: OperateMode::from(optional!("OPERATE_MODE", "public").as_str()),

            smtp: {
                if let Some(host) = optional!("SMTP_HOST")
                    && let Some(username) = optional!("SMTP_USERNAME")
                    && let Some(password) = optional!("SMTP_PASSWORD")
                    && let Some(from_email) = optional!("SMTP_FROM_EMAIL")
                    && let Ok(from_email) = from_email.parse::<Address>()
                {
                    Some(Smtp {
                        host,
                        username,
                        password,
                        from_email,
                        from_name: optional!("SMTP_FROM_NAME", "Yomuyume"),
                    })
                } else {
                    None
                }
            },
        }
    }

    pub fn get_version(&self) -> String {
        let version = env!("CARGO_PKG_VERSION");
        let mut parts = version.split('.');
        let major = parts.next().unwrap_or("0").parse().unwrap_or(0);
        let minor = parts.next().unwrap_or("0").parse().unwrap_or(0);

        format!(
            "{} - {}",
            env!("CARGO_PKG_VERSION"),
            constants::VERSION_NAMES[(major + minor - 1) as usize]
        )
    }
}
