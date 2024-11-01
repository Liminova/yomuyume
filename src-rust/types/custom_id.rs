use anyhow::{anyhow, Result};
use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomID(String);

pub type CategoryID = CustomID;
pub type TitleID = CustomID;
pub type UserID = CustomID;
pub type SessionSecret = CustomID;

impl Default for CustomID {
    fn default() -> Self {
        Self(nanoid!())
    }
}

impl CustomID {
    pub fn new() -> Self {
        Self(nanoid!())
    }

    pub fn from(id: String) -> Result<Self> {
        id.as_bytes()
            .iter()
            .try_for_each(|byte| match *byte {
                b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' => Ok(()),
                _ => Err(anyhow!("invalid nanoid string")),
            })
            .map(|_| Self(id))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
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
