use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use murmur3::murmur3_32;
use toml_edit::DocumentMut;
use tracing::warn;

use crate::AppError;

/// Metadata for a title parsed from a toml file.
///
/// All input strings are trimmed and empty string are removed.
#[derive(Debug, Clone, Default)]
pub struct TitleMetadata {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub release: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    /// hashes of the "cover" and "descriptions" fields
    pub cover_and_page_desc_hash: String,

    /// "page file name" -> "description"
    descriptions: HashMap<String, String>,

    pub path: PathBuf,
    document: DocumentMut,
}

impl TitleMetadata {
    /// Load (create if not exists) a toml file.
    pub fn load(path: &Path) -> Result<TitleMetadata, AppError> {
        let path = path.with_extension("toml");

        // create if not exists
        if !path.exists() {
            std::fs::File::create(&path).map_err(|e| {
                AppError::from(anyhow!(
                    "TitleMetadata::load: can't create toml file: {}",
                    e
                ))
            })?;
        }

        // load
        let raw = std::fs::read_to_string(&path).map_err(|e| {
            AppError::from(anyhow!(
                "TitleMetadata::load: can't read `{}`: {}",
                path.display(),
                e
            ))
        })?;

        // parse
        let mut new = TitleMetadata::default();
        new.document = raw.parse::<DocumentMut>().map_err(|e| {
            AppError::from(anyhow!(
                "TitleMetadata::load: can't parse {}: {}",
                path.display(),
                e
            ))
        })?;

        new.title = new
            .document
            .get_mut("title")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or(
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_default(),
            )
            .to_string();

        new.description = new
            .document
            .get_mut("description")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        new.cover = new
            .document
            .get_mut("cover")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        new.author = new
            .document
            .get_mut("author")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        new.release = new
            .document
            .get_mut("release")
            .and_then(|s| s.as_str())
            .and_then(|s| match s.trim().to_string().parse::<DateTime<Utc>>() {
                Ok(s) => Some(s),
                Err(e) => {
                    warn!("can't parse release date for {}: {}", path.display(), e);
                    None
                }
            });

        new.tags = new
            .document
            .get_mut("tags")
            .and_then(|s| s.as_array())
            // Array of Item -> Array of String
            // Item -> String maybe fail, String maybe empty
            .map(|tags| {
                tags.iter()
                    .filter_map(|tag| {
                        tag.as_str()
                            .map(|tag| tag.trim().to_string())
                            .filter(|tag| !tag.is_empty())
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|tags| !tags.is_empty())
            .unwrap_or_default();

        new.descriptions = new
            .document
            .get_mut("descriptions")
            .and_then(|s| s.as_table())
            // same as above, <&str, Item> -> <String, String>
            // Item -> String maybe fail, String maybe empty
            .map(|s| {
                s.iter()
                    .filter_map(|(k, v)| {
                        let k = k.trim().to_string();
                        let k = if k.is_empty() { None } else { Some(k) };
                        let v = v
                            .as_str()
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty());
                        match (k, v) {
                            (Some(k), Some(v)) => Some((k, v)),
                            _ => None,
                        }
                    })
                    .collect::<HashMap<String, String>>()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_default();

        let mut cover_and_page_desc = new
            .descriptions
            .iter()
            .map(|(k, v)| format!("{}{}", k, v))
            .collect::<Vec<_>>()
            .join("");
        cover_and_page_desc.push_str(&new.cover.clone().unwrap_or_default());

        let cover_and_page_desc_hash = match murmur3_32(&mut &cover_and_page_desc.as_bytes()[..], 0)
        {
            Ok(hash) => hash.to_string(),
            Err(e) => return Err(anyhow!("can't hash: {}", e).into()),
        };

        new.cover_and_page_desc_hash = cover_and_page_desc_hash;

        Ok(new)
    }

    // Save the metadata to the toml file
    pub fn save(&self) -> Result<(), AppError> {
        std::fs::write(&self.path, self.document.to_string()).map_err(|e| {
            AppError::from(anyhow!(
                "TitleMetadata::save: can't write metadata to {}: {}",
                self.path.display(),
                e
            ))
        })
    }

    pub fn get_page_description(&self, page_file_name: &str) -> Option<String> {
        let path = PathBuf::from(page_file_name.to_string());

        // allow users define page desc w/ or w/o file extension
        let no_ext = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let with_ext: String = path.to_string_lossy().to_string();

        if let Some(desc) = self.descriptions.get(&no_ext) {
            return Some(desc.clone());
        }
        if let Some(desc) = self.descriptions.get(&with_ext) {
            return Some(desc.clone());
        }

        None
    }
}
