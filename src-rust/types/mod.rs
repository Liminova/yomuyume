pub mod absolute_path;
pub mod category_info;
pub mod comic_info;
pub mod temp_code_purpose;
pub mod user_cache;

use serde::{Deserialize, Deserializer};

use crate::utils::macros::bail_if_empty;

macro_rules! mental_new_type {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name($inner);

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl $name {
            pub fn as_ref(&self) -> $inner {
                self.0
            }
        }
    };
}

mental_new_type!(TitleID, i64);
mental_new_type!(CategoryID, i64);
mental_new_type!(UserID, i64);
mental_new_type!(SessionID, i64);

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
