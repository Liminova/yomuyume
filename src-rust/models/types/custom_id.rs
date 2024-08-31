use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, sea_orm::DeriveValueType)]
pub struct CustomID(String);

impl Default for CustomID {
    fn default() -> Self {
        Self(nanoid!())
    }
}

impl CustomID {
    pub fn new() -> Self {
        Self(nanoid!())
    }

    pub fn from(id: String) -> Result<Self, String> {
        id.as_bytes()
            .iter()
            .try_for_each(|byte| match *byte {
                b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' => Ok(()),
                _ => Err("invalid nanoid string".to_string()),
            })
            .map(|_| Self(id))
    }
}

impl Serialize for CustomID {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

struct CustomIDVisitor;
impl<'de> serde::de::Visitor<'de> for CustomIDVisitor {
    type Value = CustomID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string only containing alphanumeric characters, - or _")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        CustomID::from(String::from(value))
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(value), &self))
    }
}

impl<'de> Deserialize<'de> for CustomID {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_string(CustomIDVisitor)
    }
}

impl From<CustomID> for String {
    fn from(id: CustomID) -> Self {
        id.0
    }
}

impl From<&CustomID> for String {
    fn from(id: &CustomID) -> Self {
        id.0.clone()
    }
}

impl From<&CustomID> for CustomID {
    fn from(id: &CustomID) -> Self {
        id.clone()
    }
}

impl std::fmt::Display for CustomID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for CustomID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// SeaORM New Type - https://www.sea-ql.org/SeaORM/docs/generate-entity/newtype/
// 1, 2, 3. satisfied by the DeriveValueType macro
// 4. If the field is Option<T>, implement sea_query::Nullable for T
impl sea_orm::sea_query::Nullable for CustomID {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}

// SeaORM wouldn't shut up about this even though we don't use auto increment
impl sea_orm::TryFromU64 for CustomID {
    fn try_from_u64(value: u64) -> Result<Self, sea_orm::DbErr> {
        match Self::from(value.to_string()) {
            Ok(id) => Ok(id),
            Err(e) => Err(sea_orm::DbErr::Custom(e)),
        }
    }
}

// to be able to use references when writing conditions, instead of cloning
impl From<&CustomID> for sea_orm::Value {
    fn from(id: &CustomID) -> Self {
        sea_orm::Value::String(Some(Box::new(id.0.clone())))
    }
}
