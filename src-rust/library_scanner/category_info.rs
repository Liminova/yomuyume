use std::path::PathBuf;

use anyhow::{Context, Result};
use quick_xml::de::from_str;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{models::prelude::CategoryID, SUPPORTED_IMAGE_FORMATS};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CategoryInfo {
    #[serde(
        rename = "ID",
        default = "ID::default",
        deserialize_with = "ID::deserializer",
        serialize_with = "ID::serializer"
    )]
    pub id: ID,
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
    #[serde(
        rename = "Cover",
        default,
        deserialize_with = "cover_deserializer",
        skip_serializing_if = "Option::is_none",
        serialize_with = "cover_serializer"
    )]
    pub cover: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ID {
    id: CategoryID,
    is_new: bool,
}

impl Default for ID {
    fn default() -> Self {
        Self {
            id: CategoryID::new(),
            is_new: true,
        }
    }
}

impl Into<CategoryID> for ID {
    fn into(self) -> CategoryID {
        self.id
    }
}

impl ID {
    fn deserializer<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
            return Ok(ID::default());
        }
        CategoryID::from(s)
            .map(|id| Self { id, is_new: false })
            .map_err(|_| serde::de::Error::custom("invalid category id"))
    }

    fn serializer<S>(id: &ID, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&id.id.to_string())
    }

    pub fn as_ref(&self) -> &CategoryID {
        &self.id
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }
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

fn cover_deserializer<'de, D: serde::Deserializer<'de>>(
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
    if !SUPPORTED_IMAGE_FORMATS.contains_key(ext.as_str()) {
        return Err(serde::de::Error::custom(format!(
            "cover file is not a supported image format: {ext}"
        )));
    }
    Ok(Some(path))
}

fn cover_serializer<S: Serializer>(
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

impl CategoryInfo {
    pub fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Ok(Self::default());
        }

        from_str::<CategoryInfo>(s).context("can't parse CategoryInfo.xml")
    }

    pub fn to_pretty_string(&self) -> Result<String> {
        let mut buffer = String::new();
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

    use quick_xml::de::from_str;
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn de_perfect() {
        let temp_dir =
            TempDir::new("test-category-info-der-perfect").expect("can't create temp dir");
        let cover_path = temp_dir.path().join("cover.jpg");
        let _ = File::create(&cover_path).expect("can't create cover file");

        let xml = format!(
            "
            <CategoryInfo>
                <Name>  Adventure    </Name>
                <Description>  Lorem Ipsum</Description>
                <Cover> {}    </Cover>
            </CategoryInfo>
        ",
            cover_path.to_string_lossy()
        );

        let category_info: CategoryInfo = from_str::<CategoryInfo>(&xml).unwrap();

        assert_eq!(category_info.name, Some("Adventure".to_string()));
        assert_eq!(category_info.description, Some("Lorem Ipsum".to_string()));
        assert_eq!(category_info.cover, Some(cover_path.clone()));
    }

    #[test]
    fn de_no_fields() {
        let xml = r#"<CategoryInfo></CategoryInfo>"#;

        let category_info: CategoryInfo = from_str::<CategoryInfo>(xml).unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
        assert_eq!(category_info.cover, None);
    }

    #[test]
    fn de_nothing_expect_err() {
        assert!(from_str::<CategoryInfo>("").is_err());
    }

    #[test]
    fn de_all_empty() {
        let xml = r#"
            <CategoryInfo>
                <Name></Name>
                <Description></Description>
                <Cover></Cover>
            </CategoryInfo>
        "#;

        let category_info: CategoryInfo = from_str::<CategoryInfo>(xml).unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
        assert_eq!(category_info.cover, None);
    }

    #[test]
    fn de_cover_not_exists() {
        let xml = r#"
            <CategoryInfo>
                <Name>Adventure</Name>
                <Description>Lorem Ipsum</Description>
                <Cover>this-is-not-exists.jpg</Cover>
            </CategoryInfo>
        "#;

        assert!(from_str::<CategoryInfo>(xml).is_err());
    }
}
