use std::{collections::HashMap, fmt::Display, path::PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use murmur3::murmur3_32;
use toml_edit::DocumentMut;
use tracing::warn;

/// Just a fancy wrapper for [`toml_edit`]
#[derive(Debug, Default)]
pub struct TitleMetadata {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub release: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    /// if != one in DB, re-scan the content file and check if the
    /// cover file is valid, and re-assign the pages' descriptions
    pub cover_and_page_desc_hash: u32,

    /// "page file name" -> "description"
    descriptions: HashMap<String, String>,

    document: DocumentMut,
}

impl TitleMetadata {
    pub fn new(title: impl ToString) -> Self {
        Self {
            title: title.to_string(),
            ..Default::default()
        }
    }

    pub fn parse(
        raw: impl ToString,
        backup_title: impl ToString + Display,
    ) -> Result<TitleMetadata> {
        // parse
        let mut new = TitleMetadata::default();
        new.document = raw.to_string().parse::<DocumentMut>().context(format!(
            "can't parse to TitleMetadata for \"{}\"",
            backup_title
        ))?;

        new.title = new
            .document
            .get_mut("title")
            .and_then(|s| s.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| backup_title.to_string())
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
                    warn!(
                        "can't parse release date for title \"{}\": {:#}",
                        new.title, e
                    );
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
        if let Some(cover) = new.cover.clone() {
            cover_and_page_desc.push_str(&cover);
        }

        new.cover_and_page_desc_hash = murmur3_32(&mut &cover_and_page_desc.as_bytes()[..], 0)
            .map_err(|e| {
                warn!(
                    "can't hash cover and descriptions attributes for\"{}\": {:#}",
                    new.title, e
                )
            })
            .unwrap_or_default();

        Ok(new)
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

impl Display for TitleMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.document)
    }
}
