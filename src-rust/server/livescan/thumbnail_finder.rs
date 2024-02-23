use super::blurhash::{Blurhash, BlurhashResult};
use crate::constants::{extended_img_formats, thumbnail_filestems};
use rayon::prelude::*;
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use tracing::error;
use zip::ZipArchive;

/// Find a valid thumbnail filename/path in a vector of filenames/paths
///
/// Doesn't check if the file actually exists or is a valid image
struct ThumbnailPathFinder<'a> {
    /// Just the filenames of the pages
    pub(super) exist_filepaths: &'a Vec<String>,
    /// Explicit name of the file to look for
    /// (with or without extension both work)
    pub(super) explicit_name: &'a Option<String>,
}

impl ThumbnailPathFinder<'_> {
    pub fn find(exist_filepaths: &Vec<String>, explicit_name: &Option<String>) -> Option<String> {
        let instance = ThumbnailPathFinder {
            exist_filepaths,
            explicit_name,
        };
        instance
            .both_explicit()
            .or_else(|| instance.explicit_name())
            .or_else(|| instance.fuzzy())
    }

    fn both_explicit(&self) -> Option<String> {
        let explicit_name = self.explicit_name.as_ref()?;
        self.exist_filepaths
            .iter()
            .find(|path| path.to_ascii_lowercase().contains(explicit_name))
            .map(|path| path.to_string())
    }

    fn explicit_name(&self) -> Option<String> {
        extended_img_formats().iter().find_map(|format| {
            let mut filepath = PathBuf::from(self.explicit_name.as_ref()?);
            filepath.set_extension(format);
            let filename = filepath.to_string_lossy().to_string();
            self.exist_filepaths
                .iter()
                .find(|path| path.to_ascii_lowercase().contains(&filename))
                .map(|path| path.to_string())
        })
    }

    fn fuzzy(&self) -> Option<String> {
        thumbnail_filestems().iter().find_map(|possible_filestem| {
            self.exist_filepaths
                .iter()
                .find(|path| {
                    let path = path.to_ascii_lowercase();
                    let satisfy_filestem = path.contains(possible_filestem);
                    let satisfy_extension = extended_img_formats()
                        .iter()
                        .any(|format| path.ends_with(format));
                    satisfy_filestem && satisfy_extension
                })
                .map(|path| path.to_string())
        })
    }
}

/// Finds a valid thumbnail file in a directory, then encode it to blurhash,
/// exhaustively, and return the first non-None result
///
/// ### Parameters
/// - parent_dir: Where to look for the thumbnail image file
/// - explicit_name: Explicit name of the file to look for (in <category>.toml)
///  (with or without extension both work)
/// - blurhash: An instance of Blurhash to encode the thumbnail
///
/// ### Returns a tuple
/// - The first second one is the full path to the thumbnail to be store in DB
/// because no, I'm not adding another field to BlurhashResult
/// - The second one is the path to the thumbnail file
pub fn thumbnail_finder(
    parent_dir: &Path,
    explicit_name: &Option<String>,
    blurhash: &Blurhash,
) -> Option<(BlurhashResult, PathBuf)> {
    let filepaths = parent_dir
        .read_dir()
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| {
            extended_img_formats().iter().any(|format| {
                entry
                    .path()
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .ends_with(format)
            })
        })
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();

    let filepaths_strs = filepaths
        .iter()
        .map(|entry| entry.to_string_lossy().to_string())
        .collect::<Vec<String>>();

    // Find a thumbnail file, blurhash-encode then return if Some(blurhash)
    let result = ThumbnailPathFinder::find(&filepaths_strs, explicit_name);
    if let Some(result) = result {
        let result = PathBuf::from(result);
        let extension = result
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let blurhash = blurhash.encode(&result, &extension);
        if let Some(blurhash) = blurhash {
            return Some((blurhash, result));
        }
    }

    // Last resort, encode all if nothing found, return first non-None
    filepaths
        .par_iter()
        .find_map_any(|path| {
            let extension = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            blurhash.encode(path, &extension)
        })
        .map(|blurhash| (blurhash, filepaths[0].clone()))
}

/// Extract a title zip, then find a valid thumbnail file in it, then encode it
/// to blurhash, exhaustively, and return the first non-None result
pub async fn title_thumbnail_finder(
    temp_dir: &PathBuf,
    title_path: &PathBuf,
    explicit_name: &Option<String>,
    blurhash: &Blurhash,
) -> Option<BlurhashResult> {
    // Creating a temp dir for the title
    let title_temp_dir = {
        let mut title_temp_dir = PathBuf::from(temp_dir);
        title_temp_dir.push(title_path.file_stem()?.to_string_lossy().to_string());
        title_temp_dir
    };

    // Extract all
    ZipArchive::new(
        File::open(title_path)
            .map_err(|e| {
                error!("error openning title: {}", e);
                e
            })
            .ok()?,
    )
    .map_err(|e| {
        error!("error reading title: {}", e);
        e
    })
    .ok()?
    .extract(&title_temp_dir)
    .map_err(|e| {
        let temp_dir = title_temp_dir.to_string_lossy();
        error!("error extracting title to {}: {}", temp_dir, e);
        e
    })
    .ok()?;

    // Find and encode thumbnail
    let result = thumbnail_finder(&title_temp_dir, explicit_name, blurhash)
        .map(|(blurhash, _)| blurhash)
        .ok_or_else(|| {
            let temp_dir = title_temp_dir.to_string_lossy();
            error!("no thumbnail found in {}", temp_dir);
            std::io::Error::new(std::io::ErrorKind::NotFound, "no thumbnail found")
        })
        .ok();

    // Delete temp dir
    let handle = tokio::spawn(async move {
        let _ = tokio::fs::remove_dir_all(&title_temp_dir).await;
    });
    std::mem::drop(handle);

    result
}
