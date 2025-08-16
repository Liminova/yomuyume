use std::sync::Arc;

use dashmap::DashMap;
use tokio::time::{Duration, sleep};

use crate::{
    structs::ids::{SessionID, UserID},
    traits::chrono_utils::ChronoUtils,
};

use chrono::{DateTime, TimeDelta, Utc};

#[derive(Debug, Clone)]
pub struct CachedUser {
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub ip_address: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,

    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct CachedUserBuilder {
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub ip_address: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
}

impl CachedUserBuilder {
    pub fn username(mut self, username: String) -> Self {
        self.username = username;
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = email;
        self
    }

    pub fn profile_picture(mut self, profile_picture: Option<String>) -> Self {
        self.profile_picture = profile_picture;
        self
    }

    pub fn ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = ip_address;
        self
    }

    pub fn updated_at(mut self, updated_at: Option<DateTime<Utc>>) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub fn verified_at(mut self, verified_at: Option<DateTime<Utc>>) -> Self {
        self.verified_at = verified_at;
        self
    }

    pub fn build(self) -> CachedUser {
        CachedUser {
            username: self.username,
            email: self.email,
            profile_picture: self.profile_picture,
            ip_address: self.ip_address,
            updated_at: self.updated_at,
            verified_at: self.verified_at,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedSession {
    pub user_id: UserID,
    pub secret: String,
    pub last_used_at: DateTime<Utc>,

    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct CachedSessionBuilder {
    pub user_id: UserID,
    pub secret: String,
    pub last_used_at: DateTime<Utc>,
}

impl CachedSessionBuilder {
    pub fn user_id(mut self, user_id: UserID) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn secret(mut self, secret: String) -> Self {
        self.secret = secret;
        self
    }

    pub fn last_used_at(mut self, last_used_at: DateTime<Utc>) -> Self {
        self.last_used_at = last_used_at;
        self
    }

    pub fn build(self) -> CachedSession {
        CachedSession {
            user_id: self.user_id,
            secret: self.secret,
            last_used_at: self.last_used_at,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct Cacher {
    pub users: Arc<DashMap<UserID, CachedUser>>,
    pub sessions: Arc<DashMap<SessionID, CachedSession>>,
}

#[derive(Debug)]
pub struct CacherBuilder {
    time_to_live: TimeDelta,
    sweep_interval: Duration,
}

impl Default for CacherBuilder {
    fn default() -> Self {
        Self {
            time_to_live: TimeDelta::hours(6),
            sweep_interval: Duration::from_secs(60 * 60 * 6),
        }
    }
}

impl CacherBuilder {
    pub fn time_to_live(mut self, delta: TimeDelta) -> Self {
        self.time_to_live = delta;
        self
    }

    pub fn sweep_intervval(mut self, duration: Duration) -> Self {
        self.sweep_interval = duration;
        self
    }

    pub fn build(self) -> Cacher {
        let cacher = Cacher {
            users: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        };

        let user_cache = cacher.users.clone();
        let session_cache = cacher.sessions.clone();

        tokio::spawn(async move {
            loop {
                user_cache
                    .retain(|_, user| user.created_at.inside(&Utc::now(), &self.time_to_live));

                session_cache.retain(|_, session| {
                    session.created_at.inside(&Utc::now(), &self.time_to_live)
                });

                sleep(self.sweep_interval).await;
            }
        });

        cacher
    }
}
