pub mod absolute_path;
pub mod category_info;
pub mod comic_info;
pub mod id;
pub mod user_cache;

use serde::{Deserialize, Deserializer};

fn option_blurhash_deserializer<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.trim().to_string();
    if s.is_empty() {
        return Ok(None);
    }
    if let Err(e) = blurhash::decode(&s, 0, 0, 0.0) {
        return Err(serde::de::Error::custom(e));
    }
    Ok(Some(s))
}
