#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub server_address: String,
    pub server_port: u16,
    pub database_url: String,
    pub library_path: String,

    pub jwt_secret: String,
    pub jwt_maxage: chrono::Duration,

    pub smtp_host: Option<String>,
    pub smtp_port: Option<usize>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_from_name: Option<String>,

    pub ffmpeg_path: Option<String>,
    pub djxl_path: Option<String>,
    pub ffmpeg_log_path: Option<String>,
    pub temp_path: String,

    pub sentence_embedding_model_path: Option<String>,
}

impl Config {
    fn get_env(key: &str, default: Option<&str>) -> String {
        match default {
            Some(val) => std::env::var(key).unwrap_or(val.to_string()),
            None => std::env::var(key).unwrap_or_else(|_| panic!("{} must be set.", key)),
        }
    }

    fn may_get(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    pub fn init() -> Self {
        let app_name = Self::get_env("APP_NAME", Some("Yomuyume"));
        let server_address = Self::get_env("SERVER_ADDRESS", Some("0.0.0.0"));
        let server_port = Self::get_env("SERVER_PORT", Some("3000"))
            .parse()
            .unwrap_or(3000);
        let database_url = Self::get_env("DATABASE_URL", Some("sqlite:./database/sqlite.db"));
        let library_path = Self::get_env("LIBRARY_PATH", Some("./library"));

        let jwt_secret = Self::get_env("JWT_SECRET", None);
        let jwt_maxage_day = Self::get_env("JWT_MAXAGE_DAY", Some("30"))
            .parse()
            .unwrap_or(30);

        let smtp_host = Self::may_get("SMTP_HOST");
        let smtp_port = Self::may_get("SMTP_PORT").map(|port| port.parse::<usize>().unwrap_or(587));
        let smtp_username = Self::may_get("SMTP_USERNAME");
        let smtp_password = Self::may_get("SMTP_PASSWORD");
        let smtp_from_email = Self::may_get("SMTP_FROM_EMAIL");
        let smtp_from_name = Self::may_get("SMTP_FROM_NAME");

        let ffmpeg_path = Self::may_get("FFMPEG_PATH");
        let djxl_path = Self::may_get("DJXL_PATH");
        let ffmpeg_log_path = Self::may_get("FFMPEG_LOG_PATH");
        let temp_path = Self::get_env("TEMP_DIR", Some("/tmp"));

        let sentence_embedding_model_path = Self::may_get("SENTENCE_EMBEDDING_MODEL_PATH");

        Self {
            library_path,
            app_name,
            server_address,
            server_port,
            database_url,

            jwt_secret,
            jwt_maxage: chrono::Duration::days(jwt_maxage_day),

            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from_email,
            smtp_from_name,

            ffmpeg_path,
            djxl_path,
            ffmpeg_log_path,
            temp_path,

            sentence_embedding_model_path,
        }
    }
}
