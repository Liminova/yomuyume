use std::path::PathBuf;

use crate::structs::absolute_path::AbsolutePath;

#[derive(Debug, Clone)]
pub struct Smtp {
    pub host: String,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen_address: String,

    pub library_path: AbsolutePath,
    pub database_url: String,
    pub reverse_proxy_ip_header: Option<String>,

    pub feature_nomedia: bool,
    pub feature_komga_oneshot: bool,
    pub feature_komga_recycle: bool,

    pub rescan_interval: tokio::time::Duration,
    pub rescan_thread: usize,
    pub snowflake_thread: usize,

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
                    "failed to parse {} as a number: {}, using {}",
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
                    "failed to parse {} as a boolean: {}, using {}",
                    $env,
                    e,
                    $default
                )
            })
            .unwrap_or_else(|_| $default)
    };
}

impl Config {
    pub fn init() -> Self {
        let library_path = AbsolutePath::from(&PathBuf::from(must!("LIBRARY_PATH")), None)
            .expect("can't convert LIBRARY_PATH to absolute");

        assert!(
            library_path.as_ref().is_dir(),
            "LIBRARY_PATH is not point to a valid directory"
        );

        Self {
            listen_address: optional!("LISTEN_ADDRESS", "0.0.0.0:3000"),

            library_path,
            database_url: must!("DATABASE_URL"),
            reverse_proxy_ip_header: optional!("REVERSE_PROXY_IP_HEADER"),

            feature_nomedia: optional!(bool: "FEATURE_NOMEDIA", false),
            feature_komga_oneshot: optional!(bool: "FEATURE_KOMGA_ONESHOT", false),
            feature_komga_recycle: optional!(bool: "FEATURE_KOMGA_RECYCLE", false),

            rescan_interval: tokio::time::Duration::from_secs(
                optional!(num: "RESCAN_INTERVAL_SECS", 6 * 60 * 60),
            ),
            rescan_thread: optional!(num: "RESCAN_THREAD", 8),
            snowflake_thread: optional!(num: "SNOWFLAKE_THREAD", 8),

            smtp: {
                if let Some(host) = optional!("SMTP_HOST")
                    && let Some(username) = optional!("SMTP_USERNAME")
                    && let Some(password) = optional!("SMTP_PASSWORD")
                    && let Some(from_email) = optional!("SMTP_FROM_EMAIL")
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
        let semver = env!("CARGO_PKG_VERSION").parse::<semver::Version>();

        if let Ok(semver) = semver {
            format!(
                "{} - {}",
                semver,
                super::constants::VERSION_NAMES[(semver.major + semver.minor - 1) as usize]
            )
        } else {
            tracing::warn!(
                "couldn't parse a semver out of Cargo.toml? defaulting to 0.0.0-unknown"
            );
            String::from("0.0.0-unknown - No Version Name")
        }
    }
}
