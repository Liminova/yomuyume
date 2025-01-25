#[derive(Debug, Clone)]
pub struct SessionToken {
    pub user_id: i64,
    pub session_secret: String,
    pub last_used_at: chrono::DateTime<chrono::Utc>,
}
