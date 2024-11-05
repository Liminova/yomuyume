use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use quick_xml::de::from_str;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::option_blurhash_deserializer;
use crate::{CATEGORY_INFO_SCHEMA, SUPPORTED_IMAGE_FORMATS};

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
    if s.is_empty() || s.to_ascii_lowercase() == "untitled" {
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
    if s.is_empty() {
        return Ok(None);
    }
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
    #[serde(
        rename = "@Path",
        default,
        deserialize_with = "Cover::path_deserializer",
        serialize_with = "Cover::path_serializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub path: Option<PathBuf>,
    #[serde(
        rename = "@Blurhash",
        default,
        deserialize_with = "option_blurhash_deserializer",
        skip_serializing_if = "Option::is_none"
    )]
    pub blurhash: Option<String>,
    #[serde(rename = "@Width", default, skip_serializing_if = "Cover::is_zero")]
    pub width: u32,
    #[serde(rename = "@Height", default, skip_serializing_if = "Cover::is_zero")]
    pub height: u32,
    #[serde(
        rename = "@ModifiedDateAtEncode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub modified_date_at_encode: Option<DateTime<Utc>>,
}

impl Cover {
    fn path_deserializer<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<PathBuf>, D::Error> {
        let s = String::deserialize(deserializer)?.trim().to_string();

        if s.is_empty() {
            return Ok(None);
        }
        let path = PathBuf::from(s);
        if !path.exists() {
            return Err(serde::de::Error::custom(format!(
                "cover file not exists: {path:?}"
            )));
        }
        if !path.is_file() {
            return Err(serde::de::Error::custom(format!(
                "cover file is not a file: {path:?}"
            )));
        }
        let ext = path
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if !SUPPORTED_IMAGE_FORMATS.contains(&ext.as_str()) {
            return Err(serde::de::Error::custom(format!(
                "cover file is not a supported image format: {ext}"
            )));
        }
        Ok(Some(path))
    }

    fn path_serializer<S: Serializer>(
        cover: &Option<PathBuf>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match cover {
            Some(cover) => {
                let s = cover.to_string_lossy().to_string().trim().to_string();
                if s.is_empty() {
                    return Err(serde::ser::Error::custom("empty string"));
                }
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none(),
        }
    }

    fn is_zero(number: &u32) -> bool {
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
    pub fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Ok(Self::default());
        }
        from_str::<CategoryInfo>(s).context("can't parse CategoryInfo.xml")
    }

    pub fn to_pretty_string(&self) -> Result<String> {
        let mut buffer = format!("{CATEGORY_INFO_SCHEMA}\n");
        let mut ser = quick_xml::se::Serializer::new(&mut buffer);
        ser.indent(' ', 4);
        self.serialize(ser)
            .context("can't serialize CategoryInfo.xml")?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempdir::TempDir;

    use super::*;
    use crate::CATEGORY_INFO_SCHEMA;

    #[test]
    fn perfect() {
        let temp_dir =
            TempDir::new("test-category-info-der-perfect").expect("can't create temp dir");
        let cover_path = temp_dir.path().join("cover.jpg");
        let _ = File::create(&cover_path).expect("can't create cover file");

        let category_info: CategoryInfo = CategoryInfo::from_str(
            &(format!(
                "
            <CategoryInfo>
                <Name>  Adventure    </Name>
                <Description>  Lorem Ipsum</Description>
                <Cover Path=\"{}\">    </Cover>
            </CategoryInfo>
        ",
                cover_path.to_string_lossy()
            )),
        )
        .unwrap();

        assert_eq!(category_info.name, Some("Adventure".to_string()));
        assert_eq!(category_info.description, Some("Lorem Ipsum".to_string()));
        assert_eq!(
            category_info.cover,
            Some(Cover {
                path: Some(cover_path.clone()),
                ..Default::default()
            })
        );

        assert_eq!(
            category_info.to_pretty_string().unwrap(),
            format!(
                "{CATEGORY_INFO_SCHEMA}
<CategoryInfo>
    <Name>Adventure</Name>
    <Description>Lorem Ipsum</Description>
    <Cover Path=\"{}\"/>
</CategoryInfo>",
                cover_path.to_string_lossy()
            )
        );
    }

    #[test]
    fn no_fields() {
        let category_info: CategoryInfo =
            from_str::<CategoryInfo>(r#"<CategoryInfo></CategoryInfo>"#).unwrap();

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

    #[test]
    fn cover_not_exists() {
        assert!(CategoryInfo::from_str(
            r#"
            <CategoryInfo>
                <Name>Adventure</Name>
                <Description>Lorem Ipsum</Description>
                <Cover Path="this-is-not-exists.jpg" />
            </CategoryInfo>
            "#
        )
        .is_err());
    }
}
