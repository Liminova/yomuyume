#![allow(clippy::ref_option)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::option_blurhash_deserializer;
use crate::utils::{config::CATEGORY_INFO_SCHEMA, macros::bail_if_empty};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CategoryInfo {
    #[serde(
        rename = "Name",
        default,
        deserialize_with = "name_deserializer",
        serialize_with = "name_serializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    #[serde(
        rename = "Description",
        default,
        deserialize_with = "description_deserializer",
        serialize_with = "description_serializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
    #[serde(rename = "Cover", default, skip_serializing_if = "Cover::all_empty")]
    pub cover: Option<Cover>,
}

fn name_deserializer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let s = String::deserialize(deserializer)?.trim().to_string();
    if s.is_empty() || s.eq_ignore_ascii_case("untitled") {
        return Ok(None);
    }
    Ok(Some(s))
}

fn name_serializer<S: Serializer>(name: &Option<String>, serializer: S) -> Result<S::Ok, S::Error> {
    match name {
        Some(name) => serializer.serialize_str(name),
        None => serializer.serialize_none(),
    }
}

fn description_deserializer<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let s = String::deserialize(deserializer)?.trim().to_string();
    bail_if_empty!(s, Ok(None));
    Ok(Some(s))
}

fn description_serializer<S: Serializer>(
    description: &Option<String>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match description {
        Some(description) => serializer.serialize_str(description),
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Cover {
    #[serde(rename = "@Path", default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(
        rename = "@Blurhash",
        default,
        deserialize_with = "option_blurhash_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub blurhash: Option<String>,
    #[serde(rename = "@Width", default, skip_serializing_if = "Cover::is_zero")]
    pub width: i32,
    #[serde(rename = "@Height", default, skip_serializing_if = "Cover::is_zero")]
    pub height: i32,
    #[serde(
        rename = "@ModifiedDateAtEncode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub modified_date_at_encode: Option<DateTime<Utc>>,
}

impl Cover {
    fn is_zero(number: &i32) -> bool {
        *number == 0
    }

    fn all_empty(cover: &Option<Cover>) -> bool {
        if let Some(cover) = cover {
            return cover.blurhash.is_none()
                && cover.width == 0
                && cover.height == 0
                && cover.modified_date_at_encode.is_none()
                && cover.path.is_none();
        }
        true
    }
}

impl CategoryInfo {
    pub fn from_str(s: &str) -> Result<Self, quick_xml::DeError> {
        bail_if_empty!(s, Ok(Self::default()));
        quick_xml::de::from_str::<CategoryInfo>(s)
    }

    pub fn to_pretty_string(&self) -> Result<String, quick_xml::errors::serialize::SeError> {
        let mut buffer = format!("{CATEGORY_INFO_SCHEMA}\n");
        let mut ser = quick_xml::se::Serializer::new(&mut buffer);
        ser.indent(' ', 4);
        self.serialize(ser)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::CATEGORY_INFO_SCHEMA;

    #[test]
    fn perfect() {
        let category_info: CategoryInfo = CategoryInfo::from_str(
            r#"<CategoryInfo>
                <Name>  Adventure    </Name>
                <Description>  Lorem Ipsum</Description>
                <Cover Path="Foo">    </Cover>
            </CategoryInfo>"#,
        )
        .unwrap();

        assert_eq!(category_info.name, Some("Adventure".to_string()));
        assert_eq!(category_info.description, Some("Lorem Ipsum".to_string()));
        assert_eq!(
            category_info.cover,
            Some(Cover {
                path: Some("Foo".to_string()),
                ..Default::default()
            })
        );
    }

    #[test]
    fn no_fields() {
        let category_info: CategoryInfo =
            quick_xml::de::from_str::<CategoryInfo>(r#"<CategoryInfo></CategoryInfo>"#).unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
        assert_eq!(category_info.cover, None);
    }

    #[test]
    fn all_empty() {
        let category_info: CategoryInfo =
            CategoryInfo::from_str(r#"<CategoryInfo><Name/><Description/><Cover/></CategoryInfo>"#)
                .unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
        assert_eq!(category_info.cover, Some(Cover::default()));

        assert_eq!(
            category_info.to_pretty_string().unwrap(),
            format!("{CATEGORY_INFO_SCHEMA}\n<CategoryInfo/>")
        );
    }
}
