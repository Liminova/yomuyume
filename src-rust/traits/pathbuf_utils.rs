use std::{collections::VecDeque, path::PathBuf};

use chrono::{DateTime, Timelike, Utc};

use crate::config::{SUPPORTED_ARCHIVE_FORMATS, SUPPORTED_IMAGE_FORMATS};

pub trait PathBufUtils {
    fn has_image_ext(&self) -> bool;
    fn has_archive_ext(&self) -> bool;
    fn scan_dir_recursively_for_image(&self, nomedia_support: bool) -> Vec<PathBuf>;
    fn has_recycle_flag(&self, feature_enabled: bool) -> bool;
    fn has_oneshot_flag(&self, feature_enabled: bool) -> bool;
    fn contains_nomedia_file(&self, feature_enabled: bool) -> bool;
    fn contains_category_info_file(&self) -> bool;
    fn last_modified(&self) -> Result<DateTime<Utc>, LastModifiedErr>;
    fn create_file_if_not_exists(&self) -> Result<(), std::io::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum LastModifiedErr {
    #[error("can't get metadata: {0:?}")]
    GetMetadataErr(std::io::Error),
    #[error("can't get modified time of file {0}: {1:?}")]
    GetModifiedErr(String, std::io::Error),
}

impl PathBufUtils for PathBuf {
    /// Check if the path (assumed to be a file) has an image extension
    fn has_image_ext(&self) -> bool {
        self.extension().is_some_and(|ext| {
            SUPPORTED_IMAGE_FORMATS.contains(&ext.to_string_lossy().to_string().as_ref())
        })
    }

    /// Check if the path (assumed to be a file) has an archive extension
    fn has_archive_ext(&self) -> bool {
        self.extension().is_some_and(|ext| {
            SUPPORTED_ARCHIVE_FORMATS.contains(&ext.to_string_lossy().to_string().as_ref())
        })
    }

    /// Scan a path (assumed to be a directory) recursively for image files
    fn scan_dir_recursively_for_image(&self, nomedia_support: bool) -> Vec<PathBuf> {
        let mut files = vec![];
        let mut queue: VecDeque<PathBuf> = VecDeque::from([self.clone()]);

        while let Some(entry) = queue.pop_back() {
            if entry.is_file() {
                if nomedia_support && entry.join(".nomedia").exists() {
                    continue;
                }
                if entry.has_image_ext() {
                    files.push(entry);
                }
                continue;
            }
            queue.push_front(entry);
        }
        files
    }

    /// Check if any of the path's components contains Komga's `_oneshot`
    fn has_oneshot_flag(&self, feature_enabled: bool) -> bool {
        feature_enabled
            && self
                .components()
                .any(|c| c.as_os_str().to_string_lossy().contains("_oneshot"))
    }

    /// Check if any of the path's components contains Komga's `#recycle`
    fn has_recycle_flag(&self, feature_enabled: bool) -> bool {
        feature_enabled
            && self
                .components()
                .any(|c| c.as_os_str().to_string_lossy().contains("#recycle"))
    }

    /// Check if the path (assumed to be a directory) contains `.nomedia`
    fn contains_nomedia_file(&self, feature_enabled: bool) -> bool {
        feature_enabled && self.join(".nomedia").exists()
    }

    /// Check if the path (assumed to be a directory) contains `CategoryInfo.xml`
    fn contains_category_info_file(&self) -> bool {
        self.join("CategoryInfo.xml").exists()
    }

    /// Get the last modified time of the path
    fn last_modified(&self) -> Result<DateTime<Utc>, LastModifiedErr> {
        Ok(DateTime::<Utc>::from(
            self.metadata()
                .map_err(LastModifiedErr::GetMetadataErr)?
                .modified()
                .map_err(|e| LastModifiedErr::GetModifiedErr(self.display().to_string(), e))?,
        )
        .with_nanosecond(0)
        .unwrap_or_default())
    }

    /// Create the file if it doesn't exist
    ///
    /// The only error is from [`std::fs::File::create`]
    fn create_file_if_not_exists(&self) -> Result<(), std::io::Error> {
        if self.exists() {
            return Ok(());
        }
        std::fs::File::create(self)?;
        Ok(())
    }
}
