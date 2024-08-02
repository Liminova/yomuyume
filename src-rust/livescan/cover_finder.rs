use crate::config::Config;

use super::blurhash::encode;
use std::{
    cmp::Ordering,
    fs::File,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

pub struct CoverFinderResult {
    pub blurhash: String,
    pub ratio: u32,
    pub image_path: PathBuf,
}

/// Finds a valid cover image in a category directory and encode it to blurhash.
/// If `configured_file` is Some(image) and encode-able, it will be used first,
/// else try every single image in the directory.
///
/// ### Parameters
/// - `config`: the app config
/// - `parent_dir`: Where to look for the cover image file
/// - `configured_file`: the file configured in the category.toml
pub fn category_cover_finder(
    config: &Config,
    parent_dir: &Path,
    configured_file: &Option<String>,
) -> Result<CoverFinderResult, String> {
    // all files in parent dir
    let mut files = parent_dir
        .read_dir()
        .map_err(|e| format!("can't read category dir: {}", e))?
        .filter_map(|entry| {
            // scan success
            let entry = entry.ok()?;
            // not a dir
            if entry.path().is_dir() {
                return None;
            }
            // extension supported
            if !config.extended_img_formats.contains(
                &entry
                    .path()
                    .extension()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .to_str()?,
            ) {
                return None;
            }
            Some(entry.path().to_string_lossy().to_string())
        })
        .collect::<Vec<String>>();

    if files.is_empty() {
        return Err("no cover found".to_string());
    }

    // sort alphabetically, keep name.contains(any of config.cover_filestems) first
    files.sort_by(|a, b| {
        let a = a.to_lowercase();
        let b = b.to_lowercase();
        for include_stem in &config.cover_filestems {
            match (a.contains(include_stem), b.contains(include_stem)) {
                (true, true) => return Ordering::Equal,
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                _ => continue,
            }
        }
        a.cmp(&b)
    });

    // try encode the configured file
    if let Some(configured_file) = configured_file {
        if files.contains(&configured_file.to_string()) {
            let configured_file = PathBuf::from(configured_file);
            match encode(config, &configured_file) {
                Ok(result) => {
                    return Ok(CoverFinderResult {
                        blurhash: result.blurhash,
                        ratio: result.ratio,
                        image_path: configured_file,
                    })
                }
                Err(e) => {
                    tracing::warn!("configured cover failed to encode: {}", e);
                }
            }
        }
    }

    // try encode all files until one succeeds
    for path in files {
        let path = PathBuf::from(path);
        match encode(config, &path) {
            Ok(result) => {
                return Ok(CoverFinderResult {
                    blurhash: result.blurhash,
                    ratio: result.ratio,
                    image_path: path,
                })
            }
            _ => continue,
        }
    }

    Err("no cover found".to_string())
}

pub enum TitleCoverFinderError {
    NoCoverFound,
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Path(std::path::StripPrefixError),
}

/// Extract the title zip, find a valid cover image then encode it to blurhash,
/// exhaustively, and return the first non-None result
pub async fn title_cover_finder(
    config: &Config,
    title_path: &PathBuf,
    configured_file: &Option<String>,
) -> Result<CoverFinderResult, TitleCoverFinderError> {
    // create tmp dir path
    let temp_dir = {
        let mut temp_dir = PathBuf::from("/tmp");
        temp_dir.push(title_path.to_string_lossy().to_string());
        temp_dir
    };

    // extract all
    ZipArchive::new(File::open(title_path).map_err(TitleCoverFinderError::Io)?)
        .map_err(TitleCoverFinderError::Zip)?
        .extract(&temp_dir)
        .map_err(TitleCoverFinderError::Zip)?;

    let mut files = temp_dir
        .read_dir()
        .map_err(TitleCoverFinderError::Io)?
        .filter_map(|entry| {
            // scan success
            let entry = entry.ok()?;
            // not a dir
            if entry.path().is_dir() {
                return None;
            }
            // extension supported
            if !config.extended_img_formats.contains(
                &entry
                    .path()
                    .extension()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .to_str()?,
            ) {
                return None;
            }
            Some(entry.path().to_string_lossy().to_string())
        })
        .collect::<Vec<String>>();

    if files.is_empty() {
        return Err(TitleCoverFinderError::NoCoverFound);
    }

    // sort alphabetically, keep name.contains(any of config.cover_filestems) first
    files.sort_by(|a, b| {
        let a = a.to_lowercase();
        let b = b.to_lowercase();
        for include_stem in &config.cover_filestems {
            match (a.contains(include_stem), b.contains(include_stem)) {
                (true, true) => return Ordering::Equal,
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                _ => continue,
            }
        }
        a.cmp(&b)
    });

    // try encode the configured file
    if let Some(configured_file) = configured_file {
        if files.contains(&configured_file.to_string()) {
            let configured_file_in_temp = temp_dir.join(configured_file);
            if configured_file_in_temp.is_file() {
                match encode(config, &configured_file_in_temp) {
                    Ok(result) => {
                        return Ok(CoverFinderResult {
                            blurhash: result.blurhash,
                            ratio: result.ratio,
                            image_path: PathBuf::from(configured_file),
                        })
                    }
                    Err(e) => {
                        tracing::warn!("configured cover failed to encode: {}", e);
                    }
                }
            }
        }
    }

    // try encode all files until one succeeds
    for file in files {
        let file_path = PathBuf::from(file);
        match encode(config, &file_path) {
            Ok(result) => {
                // strip temp dir prefix
                let image_path = match file_path.strip_prefix(&temp_dir) {
                    Ok(path) => path.to_path_buf(),
                    Err(e) => return Err(TitleCoverFinderError::Path(e)),
                };

                let _ = std::fs::remove_dir_all(&temp_dir);

                return Ok(CoverFinderResult {
                    blurhash: result.blurhash,
                    ratio: result.ratio,
                    image_path,
                });
            }
            _ => continue,
        }
    }

    Err(TitleCoverFinderError::NoCoverFound)
}
