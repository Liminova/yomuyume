use chrono::{DateTime, Utc};
use redb::TableDefinition as TableDef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    email: Option<String>,
    password_hash: Option<String>,
    profile_picture: Option<Vec<u8>>,
    verified_at: Option<DateTime<Utc>>,
}

impl redb::Value for UserInfo {
    type SelfType<'a> = UserInfo;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        if let Ok(data) = serde_json::from_slice::<UserInfo>(data) {
            data
        } else {
            unreachable!("Failed to deserialize UserInfo, database corruption?")
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        if let Ok(data) = serde_json::to_vec(value) {
            data
        } else {
            unreachable!("Failed to serialize UserInfo, memory corruption?")
        }
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("UserInfo")
    }
}

type Username = String;

const USER_INFO: TableDef<Username, UserInfo> = TableDef::new("user_info");
