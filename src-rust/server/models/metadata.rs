use std::path::PathBuf;
use toml_edit::Document;
use tracing::{debug, info, warn};

fn try_read_toml(path: &PathBuf) -> Option<Document> {
    if !path.exists() {
        if let Err(e) = std::fs::File::create(path) {
            warn!("error creating file: {}\n", e);
            return None;
        }
        info!("created file: {}\n", path.to_string_lossy());
    }

    let toml_file = match std::fs::read_to_string(path) {
        Ok(toml_file) => toml_file,
        Err(e) => {
            warn!("error reading toml file: {}\n", e);
            return None;
        }
    };

    match toml_file.parse::<Document>() {
        Ok(doc) => Some(doc),
        Err(e) => {
            warn!("error parsing toml file: {}\n", e);
            None
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TitleMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub author: Option<String>,
    pub release_date: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Per-page description
    /// 1st element is the page number
    /// 2nd element is the description
    pub descriptions: Option<Vec<(String, String)>>,
    doc: Document,
    path: PathBuf,
}

impl TitleMetadata {
    pub async fn from(path: &PathBuf) -> Self {
        let mut new = Self::default();

        match try_read_toml(path) {
            Some(doc) => new.doc = doc,
            None => return new,
        }
        new.title = new.parse_string("title");
        new.description = new.parse_string("description");
        new.author = new.parse_string("author");
        new.tags = new.parse_array("tags");
        new.thumbnail = new.parse_string("thumbnail");
        new.release_date = new.parse_string("release");
        new.descriptions = new.parse_table("descriptions");
        new.path = path.clone();

        new
    }

    fn parse_string(&self, key: &str) -> Option<String> {
        self.doc
            .get(key)
            .and_then(|value| value.as_str().map(|s| s.to_string()))
    }

    fn parse_array(&self, key: &str) -> Option<Vec<String>> {
        self.doc.get(key).and_then(|value| {
            value.as_array().map(|a| {
                a.iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect()
            })
        })
    }

    fn parse_table(&self, key: &str) -> Option<Vec<(String, String)>> {
        let item = self
            .doc
            .get(key)
            .ok_or_else(|| {
                debug!("{} doesn't exist in toml file", key);
            })
            .ok()?;
        let mut result = Vec::new();
        if let Some((k, v)) = item.as_table()?.iter().next() {
            let page_number = k.to_string();
            let description = v.as_str().unwrap_or_default().to_string();
            result.push((page_number, description));
        }
        Some(result)
    }

    /// Return the description that matches the page filename
    pub fn get_page_desc(&self, path: &str) -> Option<String> {
        let path = PathBuf::from(path);
        let no_ext = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let with_ext = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        self.descriptions
            .iter()
            .flatten()
            .find(|(page_number, _)| page_number == no_ext || page_number == with_ext)
            .map(|(_, description)| description.clone())
    }

    pub fn set_thumbnail(&mut self, value: String) {
        self.doc["thumbnail"] = toml_edit::value(&value);
        self.thumbnail = Some(value);
        if let Err(e) = std::fs::write(&self.path, self.doc.to_string()) {
            warn!("error writing toml file: {}\n", e);
        }
    }

    // TODO: implement modify from client in future

    // fn get_mut_tag_list(&mut self) -> Option<&mut toml_edit::Array> {
    //     let mutable_list = self
    //         .doc
    //         .entry(OtherFields::Tags.as_ref())
    //         .or_insert_with(|| toml_edit::value(toml_edit::Array::new()));
    //     mutable_list.as_array_mut()
    // }

    // pub async fn add_tag(&mut self, path: &Path, values: Vec<String>) -> Option<()> {
    //     if !path.is_file() {
    //         warn!("{} doesn't exist to add a tag", path.to_string_lossy());
    //         return None;
    //     }

    //     let mut_tags = self.get_mut_tag_list()?;
    //     for value in values {
    //         mut_tags.push(value);
    //     }
    //     Some(())
    // }

    // pub async fn remove_tag(&mut self, path: &Path, value: String) -> Option<()> {
    //     if !path.is_file() {
    //         warn!("{} doesn't exist to remove a tag", path.to_string_lossy());
    //         return None;
    //     }
    //     let mut_tags = self.get_mut_tag_list()?;
    //     let index = mut_tags
    //         .iter()
    //         .position(|v| v.as_str().unwrap_or_default() == value);
    //     if let Some(index) = index {
    //         mut_tags.remove(index);
    //     }
    //     Some(())
    // }
}

#[derive(Debug, Clone, Default)]
pub struct CategoryMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub id: Option<String>,
    doc: Document,
    path: PathBuf,
}

impl CategoryMetadata {
    pub async fn from(path: &PathBuf) -> Self {
        let mut new = Self::default();

        match try_read_toml(path) {
            Some(doc) => new.doc = doc,
            None => return new,
        }

        new.name = new.parse_string("name");
        new.description = new.parse_string("description");
        new.thumbnail = new.parse_string("thumbnail");
        new.id = new.parse_string("id");
        new.path = path.clone();

        new
    }

    fn parse_string(&self, key: &str) -> Option<String> {
        self.doc
            .get(key)
            .and_then(|value| value.as_str().map(|s| s.to_string()))
    }

    pub fn set_id(&mut self, value: String) {
        debug!("set id to {}, {:?}", value, self.path);
        self.doc["id"] = toml_edit::value(&value);
        self.id = Some(value);
        if let Err(e) = std::fs::write(&self.path, self.doc.to_string()) {
            warn!("error writing toml file: {}\n", e);
        }
    }
}
