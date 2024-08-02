use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tracing::info;

use crate::models::prelude::CategoryID;

// Read a toml file, create it if not exists
fn try_read_toml(path: &Path) -> Result<String, String> {
    if !path.exists() {
        if let Err(e) = std::fs::File::create(path) {
            return Err(format!("can't create toml file: {}", e));
        }
        info!("created: {}\n", &path.to_string_lossy());
        return Ok(String::new());
    }
    match std::fs::read_to_string(path) {
        Ok(raw) => Ok(raw),
        Err(e) => Err(format!("can't read toml file: {}", e)),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TitleMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub author: Option<String>,
    pub release: Option<String>,
    pub tags: Option<Vec<String>>,

    /// "page file name" = "description"
    pub descriptions: Option<HashMap<String, String>>,

    #[serde(skip)]
    pub path: PathBuf,
}

impl TitleMetadata {
    /// Create a new TitleMetadata from a toml file. The extension
    /// is automatically added to the path.
    pub fn from(path: &Path) -> Result<TitleMetadata, String> {
        let path = path.with_extension("toml");
        let raw_data = match try_read_toml(&path) {
            Ok(raw) => raw,
            Err(e) => return Err(e),
        };
        let mut new = match toml::from_str::<TitleMetadata>(&raw_data) {
            Ok(metadata) => metadata,
            Err(e) => return Err(format!("can't parse toml file: {}", e)),
        };
        new.path = path;
        Ok(new)
    }

    /// Get the description of a page inside the descriptions field
    pub fn get_page_desc(&self, path: &str) -> Option<String> {
        let path = PathBuf::from(path);

        let no_ext = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let with_ext: String = path.to_string_lossy().to_string();

        if let Some(descs) = &self.descriptions {
            if let Some(desc) = descs.get(&no_ext) {
                return Some(desc.clone());
            }
            if let Some(desc) = descs.get(&with_ext) {
                return Some(desc.clone());
            }
        }
        None
    }

    /// Save the cover path to the metadata file
    pub fn set_cover(&mut self, value: String) -> Result<(), String> {
        self.cover = Some(value);
        let toml_string: String = match toml::to_string(self) {
            Ok(s) => s,
            Err(e) => return Err(format!("can't convert to toml: {}", e)),
        };
        match std::fs::write(&self.path, toml_string) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("can't write toml to file: {}", e)),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryMetadata {
    pub id: Option<CategoryID>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,

    #[serde(skip)]
    pub path: PathBuf,
}

impl CategoryMetadata {
    /// Create a new CategoryMetadata from a toml file. The extension
    /// is automatically added to the path.
    pub fn from(path: &Path) -> Result<CategoryMetadata, String> {
        let path = path.with_extension("toml");
        let raw_data = match try_read_toml(&path) {
            Ok(raw) => raw,
            Err(e) => return Err(e),
        };
        let mut new = match toml::from_str::<CategoryMetadata>(&raw_data) {
            Ok(metadata) => metadata,
            Err(e) => return Err(format!("can't parse toml file: {}", e)),
        };
        new.path = path;
        Ok(new)
    }

    /// Save the ID to the metadata file
    pub fn set_id(&mut self, value: CategoryID) -> Result<(), String> {
        self.id = Some(value);
        let toml_string: String = match toml::to_string(self) {
            Ok(s) => s,
            Err(e) => return Err(format!("can't convert to toml: {}", e)),
        };

        match std::fs::write(&self.path, toml_string) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("can't write toml to file: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests_title {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn empty() {
        let path = Path::new("test-title-empty.toml");
        let metadata = TitleMetadata::from(path).unwrap();

        assert_eq!(metadata.title, None);
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.cover, None);
        assert_eq!(metadata.author, None);
        assert_eq!(metadata.release, None);
        assert_eq!(metadata.tags, None);
        assert_eq!(metadata.descriptions, None);

        assert!(metadata.path.exists());
        remove_file("test-title-empty.toml").unwrap();
    }

    #[test]
    fn set_cover() {
        let path = Path::new("test-title-cover.toml");
        let mut metadata = TitleMetadata::from(path).unwrap();
        metadata.set_cover("test.png".to_string()).unwrap();

        let metadata_file_content = std::fs::read_to_string(&metadata.path).unwrap();
        assert_eq!(metadata_file_content.contains("cover = \"test.png\""), true);

        assert!(metadata.path.exists());
        remove_file("test-title-cover.toml").unwrap();
    }

    #[test]
    fn path_no_ext() {
        let path = Path::new("test-title-no-ext");
        let metadata = TitleMetadata::from(path).unwrap();

        assert_eq!(metadata.path, Path::new("test-title-no-ext.toml"));

        assert!(metadata.path.exists());
        remove_file("test-title-no-ext.toml").unwrap();
    }

    #[test]
    fn path_overwrite_ext() {
        let path = Path::new("test-title-overwrite-ext.any");
        let metadata = TitleMetadata::from(path).unwrap();

        assert_eq!(metadata.path, Path::new("test-title-overwrite-ext.toml"));

        assert!(metadata.path.exists());
        remove_file("test-title-overwrite-ext.toml").unwrap();
    }
}

#[cfg(test)]
mod test_category {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn empty() {
        let path = Path::new("test-category-empty.toml");
        let metadata = CategoryMetadata::from(path).unwrap();

        assert_eq!(metadata.id, None);
        assert_eq!(metadata.name, None);
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.cover, None);

        assert!(metadata.path.exists());
        remove_file("test-category-empty.toml").unwrap();
    }

    #[test]
    fn path_no_ext() {
        let path = Path::new("test-category-no-ext");
        let metadata = TitleMetadata::from(path).unwrap();

        assert_eq!(metadata.path, Path::new("test-category-no-ext.toml"));

        assert!(metadata.path.exists());
        remove_file("test-category-no-ext.toml").unwrap();
    }

    #[test]
    fn path_overwrite_ext() {
        let path: &Path = Path::new("test-category-overwrite-ext.any");
        let metadata = TitleMetadata::from(path).unwrap();

        assert_eq!(metadata.path, Path::new("test-category-overwrite-ext.toml"));

        assert!(metadata.path.exists());
        remove_file("test-category-overwrite-ext.toml").unwrap();
    }

    #[test]
    fn set_id() {
        let path = Path::new("test-category-set-id.toml");
        let mut metadata = CategoryMetadata::from(path).unwrap();

        let category_id = CategoryID::new();
        metadata.set_id(category_id.clone()).unwrap();

        let metadata_file_content = std::fs::read_to_string(&metadata.path).unwrap();

        assert!(metadata_file_content.contains(format!("id = \"{}\"", category_id).as_str()));
        remove_file("test-category-set-id.toml").unwrap();
    }
}
