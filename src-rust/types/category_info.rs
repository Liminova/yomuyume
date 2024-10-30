use std::path::PathBuf;

use anyhow::{Context, Result};
use quick_xml::de::from_str;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{models::prelude::CategoryID, CATEGORY_INFO_SCHEMA, SUPPORTED_IMAGE_FORMATS};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CategoryInfo {
    #[serde(
        rename = "ID",
        default,
        deserialize_with = "id_deserializer",
        serialize_with = "id_serializer"
    )]
    pub id: CategoryID,
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

}

fn id_deserializer<'de, D>(deserializer: D) -> Result<CategoryID, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
        return Ok(CategoryID::new());
        }
    CategoryID::from(s).map_err(serde::de::Error::custom)
    }

fn id_serializer<S>(id: &CategoryID, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
    serializer.serialize_str(&id.to_string())
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

        let category_info: CategoryInfo = CategoryInfo::from_str(&xml).unwrap();

        assert_eq!(category_info.name, Some("Adventure".to_string()));
        assert_eq!(category_info.description, Some("Lorem Ipsum".to_string()));
        assert_eq!(category_info.cover, Some(cover_path.clone()));

        assert_eq!(
            category_info.to_pretty_string().unwrap(),
            format!(
                "{CATEGORY_INFO_SCHEMA}
<CategoryInfo>
    <ID>{}</ID>
    <Name>Adventure</Name>
    <Description>Lorem Ipsum</Description>
    <Cover>{}</Cover>
</CategoryInfo>",
                category_info.id.as_ref(),
                cover_path.to_string_lossy()
            )
        );
    }

    #[test]
    fn no_fields() {
        let xml = r#"<CategoryInfo></CategoryInfo>"#;

        let category_info: CategoryInfo = from_str::<CategoryInfo>(xml).unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
        assert_eq!(category_info.cover, None);

        assert_eq!(
            category_info.to_pretty_string().unwrap(),
            format!(
                "{CATEGORY_INFO_SCHEMA}\n<CategoryInfo>\n    <ID>{}</ID>\n</CategoryInfo>",
                category_info.id.as_ref()
            )
        );
    }

    #[test]
    fn empty_string_expect_default() {
        assert!(CategoryInfo::from_str("").is_ok());
    }

    #[test]
    fn all_empty() {
        let xml = r#"
            <CategoryInfo>
                <Name></Name>
                <Description></Description>
                <Cover></Cover>
            </CategoryInfo>
        "#;

        let category_info: CategoryInfo = CategoryInfo::from_str(xml).unwrap();

        assert_eq!(category_info.name, None);
        assert_eq!(category_info.description, None);
        assert_eq!(category_info.cover, None);

        assert_eq!(
            category_info.to_pretty_string().unwrap(),
            format!(
                "{CATEGORY_INFO_SCHEMA}\n<CategoryInfo>\n    <ID>{}</ID>\n</CategoryInfo>",
                category_info.id.as_ref()
            )
        );
    }

    #[test]
    fn cover_not_exists() {
        let xml = r#"
            <CategoryInfo>
                <Name>Adventure</Name>
                <Description>Lorem Ipsum</Description>
                <Cover>this-is-not-exists.jpg</Cover>
            </CategoryInfo>
        "#;

        assert!(from_str::<CategoryInfo>(xml).is_err());
    }

    #[test]
    fn id() {
        let xml = r#"
            <CategoryInfo>
                <ID>KpvE_vralCwx5HA_4B9y8</ID>
            </CategoryInfo>"#;
        let category_info: CategoryInfo = CategoryInfo::from_str(xml).unwrap();
        assert_eq!(
            category_info.id,
            CategoryID::from("KpvE_vralCwx5HA_4B9y8".to_string()).unwrap()
        );

        let xml = r#"
            <CategoryInfo>
                <ID></ID>
            </CategoryInfo>"#;
        let category_info: CategoryInfo = CategoryInfo::from_str(xml).unwrap();

        assert_eq!(
            category_info.to_pretty_string().unwrap(),
            format!(
                "{CATEGORY_INFO_SCHEMA}\n<CategoryInfo>\n    <ID>{}</ID>\n</CategoryInfo>",
                category_info.id.as_ref()
            )
        );
    }
}
