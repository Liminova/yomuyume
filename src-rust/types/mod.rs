pub mod category_info;
pub mod comic_info;
pub mod custom_id;
pub mod temp_code_purpose;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer};

fn option_datetime_deserializer<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.trim().to_string();
    if s.is_empty() {
        return Ok(None);
    }

    let native_datetime = s
        .parse::<NaiveDateTime>()
        .map_err(|e| serde::de::Error::custom(format!("can't parse as NaiveDateTime: {e}")))?;

    let local_datetime = Local
        .from_local_datetime(&native_datetime)
        .single()
        .ok_or_else(|| serde::de::Error::custom("invalid datetime"))?;

    Ok(Some(local_datetime.to_utc()))
}

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
    Ok(Some(s.to_string()))
}
