use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{models::prelude::CategoryID, SUPPORTED_IMAGE_FORMATS};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CategoryInfo {
    #[serde(default, rename = "ID", deserialize_with = "id_deserializer")]
    pub id: Option<CategoryID>,
    #[serde(
        rename = "Name",
        default,
        deserialize_with = "Name::deserializer",
        serialize_with = "Name::serializer",
        skip_serializing_if = "Name::is_untitled"
    )]
    pub name: Name,
    #[serde(
        rename = "Description",
        default,
        deserialize_with = "Description::deserializer",
        skip_serializing_if = "Description::is_empty",
        serialize_with = "Description::serializer"
    )]
    pub description: Description,
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
pub struct Name(String);

impl Default for Name {
    fn default() -> Self {
        Name("Untitled".to_string())
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.to_string())
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    fn deserializer<'de, D>(deserializer: D) -> Result<Name, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
            return Ok(Name("Untitled".to_string()));
        }
        Ok(Name(s))
    }

    fn serializer<S>(name: &Name, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&name.0)
    }

    fn is_untitled(&self) -> bool {
        self.0 == "Untitled"
    }
}

fn id_deserializer<'de, D>(deserializer: D) -> Result<Option<CategoryID>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?.trim().to_string();
    if s.is_empty() {
        return Ok(None);
    }
    CategoryID::from(s)
        .map_err(|_| serde::de::Error::custom("invalid category id"))
        .map(Some)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Description(Option<String>);

impl From<&str> for Description {
    fn from(value: &str) -> Self {
        let s = value.trim().to_string();
        if s.is_empty() {
            return Description(None);
        }
        Description(Some(s))
    }
}

impl From<String> for Description {
    fn from(value: String) -> Self {
        let s = value.trim().to_string();
        if s.is_empty() {
            return Description(None);
        }
        Description(Some(s))
    }
}

impl Description {
    fn deserializer<'de, D>(deserializer: D) -> Result<Description, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.trim().to_string();
        if s.is_empty() {
            return Ok(Description::default());
        }
        Ok(Description::from(s))
    }
    fn serializer<S>(description: &Description, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref s) = description.0 {
            return serializer.serialize_str(s);
        }
        serializer.serialize_none()
    }

    fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn clone_inner(&self) -> Option<String> {
        self.0.clone()
    }
}

fn cover_deserializer<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
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

fn cover_serializer<S>(cover: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
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

        assert_eq!(category_info.name, Name::from("Adventure"));
        assert_eq!(category_info.description, Description::from("Lorem Ipsum"));
        assert_eq!(category_info.cover, Some(cover_path.clone()));
    }

    #[test]
    fn de_no_fields() {
        let xml = r#"<CategoryInfo></CategoryInfo>"#;

        let category_info: CategoryInfo = from_str::<CategoryInfo>(xml).unwrap();

        assert_eq!(category_info.name, Name::from("Untitled"));
        assert_eq!(category_info.description, Description::default());
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

        assert_eq!(category_info.name, Name::from("Untitled"));
        assert_eq!(category_info.description, Description::default());
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
