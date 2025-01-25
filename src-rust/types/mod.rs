pub mod absolute_path;
pub mod category_info;
pub mod comic_info;
pub mod temp_code_purpose;
pub mod user_cache;

use serde::{Deserialize, Deserializer};

use crate::utils::macros::bail_if_empty;

pub type TitleID = i64;
pub type CategoryID = i64;
pub type UserID = i64;

fn option_blurhash_deserializer<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.trim().to_string();
    bail_if_empty!(s, Ok(None));
    if let Err(e) = blurhash::decode(&s, 0, 0, 0.0) {
        return Err(serde::de::Error::custom(e));
    }
    Ok(Some(s))
}
