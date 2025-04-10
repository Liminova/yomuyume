use std::path::PathBuf;

use crate::structs::absolute_path::AbsolutePath;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub library_path: AbsolutePath,
    pub concurrent_scan_tasks: usize,
    pub snowflake_id_thread_count: usize,

    pub listen_address: String,
    pub server_port: u16,
    pub database_url: String,
    pub reverse_proxy_ip_header: Option<String>,

    pub smtp_host: Option<String>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_from_name: Option<String>,
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
            .unwrap_or_else(|_| $default)
    };
}

impl Config {
    pub fn init() -> Self {
        let library_path = AbsolutePath::from(&PathBuf::from(must!("LIBRARY_PATH")), None)
            .expect("can't convert LIBRARY_PATH to absolute");

        assert!(library_path.as_ref().is_dir());

        Self {
            app_name: optional!("APP_NAME", "Yomuyume"),
            library_path,
            concurrent_scan_tasks: optional!(num: "CONCURRENT_SCAN_TASKS", 8),
            snowflake_id_thread_count: optional!(num: "SNOWFLAKE_ID_THREAD_COUNT", 8),

            listen_address: optional!("LISTEN_ADDRESS", "0.0.0.0"),
            server_port: optional!(num: "SERVER_PORT", 3000),
            database_url: must!("DATABASE_URL"),
            reverse_proxy_ip_header: optional!("REVERSE_PROXY_IP_HEADER"),

            smtp_host: optional!("SMTP_HOST"),
            smtp_username: optional!("SMTP_USERNAME"),
            smtp_password: optional!("SMTP_PASSWORD"),
            smtp_from_email: optional!("SMTP_FROM_EMAIL"),
            smtp_from_name: optional!("SMTP_FROM_NAME"),
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
