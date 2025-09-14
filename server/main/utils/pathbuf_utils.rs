use std::{collections::VecDeque, path::PathBuf};

use chrono::{DateTime, Timelike, Utc};
use tracing::warn;

use crate::utils::{
    archive_file::{ArchiveFile, ArchiveFileError},
    category_info::CategoryInfo,
    comic_info::ComicInfo,
    constants::{CATEGORY_INFO, COMIC_INFO, SUPPORTED_ARCHIVE_FORMATS, SUPPORTED_IMAGE_FORMATS},
};

#[derive(Debug, thiserror::Error)]
pub enum LastModifiedError {
    #[error("can't get metadata: {0:?}")]
    GetMetadataErr(std::io::Error),
    #[error("can't get modified time of file {0}: {1:?}")]
    GetModifiedErr(String, std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ReadComicInfoError {
    #[error("can't read from disk: {0:?}")]
    Read(#[from] std::io::Error),
    #[error("can't decode: {0:?}")]
    Decode(#[from] quick_xml::DeError),
    #[error("archive error: {0}")]
    ArchiveFileError(#[from] ArchiveFileError),
    #[error("expected ComicInfo.xml to be a file")]
    ExpectComicInfoFile,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadCategoryInfoError {
    #[error("can't read from disk: {0:?}")]
    Read(#[from] std::io::Error),
    #[error("can't decode: {0:?}")]
    Decode(#[from] quick_xml::DeError),
    #[error("archive error: {0}")]
    ArchiveFileError(#[from] ArchiveFileError),
    #[error("expected CategoryInfo.xml to be a file")]
    ExpectCategoryInfoFile,
    #[error("expected the path to be a directory")]
    ExpectPathDir,
}

pub trait PathBufUtils {
    fn has_image_ext(&self) -> bool;
    fn has_archive_ext(&self) -> bool;
    fn scan_dir_recursively_for_image(&self, nomedia_support: bool) -> Vec<PathBuf>;
    fn has_recycle_flag(&self, feature_enabled: bool) -> bool;
    fn has_oneshot_flag(&self, feature_enabled: bool) -> bool;
    fn contains_nomedia_file(&self, feature_enabled: bool) -> bool;
    fn last_modified(&self) -> Result<DateTime<Utc>, LastModifiedError>;
    fn read_category_info(&self) -> Result<CategoryInfo, ReadCategoryInfoError>;
    fn read_comic_info_from_dir(&self) -> Result<ComicInfo, ReadComicInfoError>;
    fn read_comic_info_from_archive(&self) -> Result<ComicInfo, ReadComicInfoError>;
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

    /// Get the last modified time of the path
    fn last_modified(&self) -> Result<DateTime<Utc>, LastModifiedError> {
        Ok(DateTime::<Utc>::from(
            self.metadata()
                .map_err(LastModifiedError::GetMetadataErr)?
                .modified()
                .map_err(|e| LastModifiedError::GetModifiedErr(self.display().to_string(), e))?,
        )
        .with_nanosecond(0)
        .unwrap_or_default())
    }

    fn read_category_info(&self) -> Result<CategoryInfo, ReadCategoryInfoError> {
        if self.is_file() {
            return Err(ReadCategoryInfoError::ExpectPathDir);
        }
        let category_info_path = self.join(CATEGORY_INFO);
        if !category_info_path.exists() {
            return Ok(CategoryInfo::default());
        }
        if category_info_path.is_dir() {
            warn!(
                "Expected file but found directory: {}",
                category_info_path.display()
            );
            return Err(ReadCategoryInfoError::ExpectCategoryInfoFile);
        }

        Ok(CategoryInfo::from_str(&std::fs::read_to_string(
            category_info_path,
        )?)?)
    }

    fn read_comic_info_from_dir(&self) -> Result<ComicInfo, ReadComicInfoError> {
        let comic_info_path = self.join(COMIC_INFO);
        if !comic_info_path.exists() {
            return Ok(ComicInfo::default());
        }
        if comic_info_path.is_dir() {
            warn!(
                "Expected file but found directory: {}",
                comic_info_path.display()
            );
            return Err(ReadComicInfoError::ExpectComicInfoFile);
        }

        Ok(ComicInfo::from_str(&std::fs::read_to_string(
            comic_info_path,
        )?)?)
    }

    fn read_comic_info_from_archive(&self) -> Result<ComicInfo, ReadComicInfoError> {
        self.read_file_from_archive(COMIC_INFO)
            .map_err(ReadComicInfoError::ArchiveFileError)
            .map(|b| String::from(String::from_utf8_lossy(&b)))
            .and_then(|s| ComicInfo::from_str(&s).map_err(ReadComicInfoError::Decode))
    }
}
