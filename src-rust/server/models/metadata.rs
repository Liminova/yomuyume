use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tracing::{debug, error, info};

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
    /// The extension of the metadata file is always .toml
    pub fn from(path: &Path) -> TitleMetadata {
        let path = path.with_extension("toml");

        let mut new = TitleMetadata::default();
        let raw_data = match try_read_toml(&path) {
            Ok(raw) => raw,
            Err(e) => {
                error!("can't read toml file: {}", e);
                return new;
            }
        };
        match toml::from_str::<TitleMetadata>(&raw_data) {
            Ok(metadata) => new = metadata,
            Err(e) => {
                error!("can't parse toml file: {}", e);
            }
        };
        new.path = path;
        new
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
    pub fn set_cover(&mut self, value: String) {
        debug!("new cover: {}", value);
        self.cover = Some(value);
        let toml_string: String = match toml::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                error!("can't convert to toml: {}", e);
                return;
            }
        };
        if let Err(e) = std::fs::write(&self.path, toml_string) {
            error!("can't write toml to file: {}", e);
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryMetadata {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,

    #[serde(skip)]
    pub path: PathBuf,
}

impl CategoryMetadata {
    pub fn from(path: &Path) -> CategoryMetadata {
        let path = path.with_extension("toml");

        let mut new = CategoryMetadata::default();
        let raw_data = match try_read_toml(&path) {
            Ok(raw) => raw,
            Err(e) => {
                error!("can't read toml file: {}", e);
                return new;
            }
        };
        match toml::from_str::<CategoryMetadata>(&raw_data) {
            Ok(metadata) => new = metadata,
            Err(e) => {
                error!("can't parse toml file: {}", e);
            }
        };
        new.path = path;
        new
    }

    /// Save the ID to the metadata file
    pub fn set_id(&mut self, value: String) {
        debug!("new id: {}", value);
        self.id = Some(value);
        let toml_string: String = match toml::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                error!("can't convert to toml: {}\n", e);
                return;
            }
        };

        if let Err(e) = std::fs::write(&self.path, toml_string) {
            error!("can't write toml to file: {}\n", e);
        }
    }
}
