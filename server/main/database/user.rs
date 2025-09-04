use chrono::{DateTime, Utc};
use redb::{MultimapTableDefinition as MultimapTableDef, TableDefinition as TableDef};
use redb_macros::{RedbJsonValue, key_function};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::database::content::TitleKey;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, RedbJsonValue)]
pub struct UserInfo {
    pub email: String,
    pub name: Option<String>,
    pub password_hash: String,
    pub profile_picture: Option<Vec<u8>>,
    pub verified_at: Option<DateTime<Utc>>,
}

pub type UserID = String;
pub type UserEmail = String;

pub const USERS: TableDef<UserID, UserInfo> = TableDef::new("user_info");

pub const USER_EMAIL_TO_ID: TableDef<UserEmail, UserID> = TableDef::new("user_email_to_id");

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, RedbJsonValue)]
pub struct Session {
    pub device: Option<String>,
    pub session_secret: String,
}

impl redb::Key for Session {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        use serde_json::from_slice;

        if let Ok(data1) = from_slice::<Session>(data1)
            && let Ok(data2) = from_slice::<Session>(data2)
        {
            return data1.session_secret.cmp(&data2.session_secret);
        }
        std::cmp::Ordering::Equal
    }
}

pub const SESSIONS: MultimapTableDef<UserID, Session> = MultimapTableDef::new("sessions");

type CollectionID = String;

pub const COLLECTIONS: TableDef<UserID, Vec<CollectionID>> = TableDef::new("collections");

pub const COLLECTION_NAMES: TableDef<CollectionID, String> = TableDef::new("collection_names");

#[key_function]
pub type CollectionContentKey = (UserID, CollectionID);

pub const COLLECTION_CONTENT: MultimapTableDef<CollectionContentKey, TitleKey> =
    MultimapTableDef::new("collection_content");

#[derive(Debug, Serialize, Deserialize, RedbJsonValue)]
pub struct ForgotPassword {
    pub created_at: DateTime<Utc>,
    pub code: String,
}

pub const FORGOT_PASSWORD: TableDef<UserID, ForgotPassword> = TableDef::new("forgot_password");

#[key_function]
type ProgressKey = (UserID, TitleKey);

pub const PROGRESS: MultimapTableDef<ProgressKey, u32> = MultimapTableDef::new("progress");
