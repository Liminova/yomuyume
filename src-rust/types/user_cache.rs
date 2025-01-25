use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct UserCache {
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub ip_address: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub session_secret: String,
    pub ss_token_last_used_at: DateTime<Utc>,
}
