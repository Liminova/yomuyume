use nanoid::nanoid;

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
        for byte in id.as_bytes() {
            match *byte {
                b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' => (),
                _ => return Err("invalid nanoid string".to_string()),
            }
        }
        Ok(Self(id))
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl serde::Serialize for CustomID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

struct CustomIDVisitor;
impl<'de> serde::de::Visitor<'de> for CustomIDVisitor {
    type Value = CustomID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string only containing alphanumeric characters, - or _")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        CustomID::from(String::from(value))
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(value), &self))
    }
}

impl<'de> serde::Deserialize<'de> for CustomID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(CustomIDVisitor)
    }
}

impl sea_orm::TryFromU64 for CustomID {
    fn try_from_u64(value: u64) -> Result<Self, sea_orm::DbErr> {
        match CustomID::from(value.to_string()) {
            Ok(id) => Ok(id),
            Err(_) => Err(sea_orm::DbErr::ConvertFromU64("invalid nanoid string")),
        }
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

impl From<&CustomID> for sea_orm::Value {
    fn from(id: &CustomID) -> Self {
        sea_orm::Value::String(Some(Box::new(id.0.clone())))
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
