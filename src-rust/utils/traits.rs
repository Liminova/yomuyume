use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Timelike, Utc};

use crate::config::{SUPPORTED_ARCHIVE_FORMATS, SUPPORTED_IMAGE_FORMATS};

pub trait IteratorExt: Iterator {
    /// Same as [`find_map`] but use [`Result<B>`] instead of [`Option<B>`]
    ///
    /// [`find_map`]: Iterator::find_map
    fn try_find_map<B, F>(self, mut f: F) -> Result<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Result<B>,
    {
        let mut error = anyhow!("Item not found");
        self.filter_map(|i| {
            f(i).map_err(|e| {
                error = e;
            })
            .ok()
        })
        .next()
        .ok_or(error)
    }
}

impl<I: Iterator> IteratorExt for I {}

pub trait PathBufUtils {
    fn has_image_ext(&self) -> bool;
    fn has_archive_ext(&self) -> bool;
    fn scan_dir_recursively_for_image(&self, nomedia_support: bool) -> Vec<PathBuf>;
    fn has_recycle_flag(&self, feature_enabled: bool) -> bool;
    fn has_oneshot_flag(&self, feature_enabled: bool) -> bool;
    fn contains_nomedia_file(&self, feature_enabled: bool) -> bool;
    fn contains_category_info_file(&self) -> bool;
    fn last_modified(&self) -> Result<DateTime<Utc>>;
}

impl PathBufUtils for PathBuf {
    /// Check if the path (assumed to be a file) has an image extension
    fn has_image_ext(&self) -> bool {
        self.extension()
            .map(|ext| {
                SUPPORTED_IMAGE_FORMATS.contains(&ext.to_string_lossy().to_string().as_ref())
            })
            .unwrap_or(false)
    }

    /// Check if the path (assumed to be a file) has an archive extension
    fn has_archive_ext(&self) -> bool {
        self.extension()
            .map(|ext| {
                SUPPORTED_ARCHIVE_FORMATS.contains(&ext.to_string_lossy().to_string().as_ref())
            })
            .unwrap_or(false)
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
    fn last_modified(&self) -> Result<DateTime<Utc>> {
        self.metadata()
            .map_err(|e| anyhow!("Can't get metadata: {}", e))?
            .modified()
            .map_err(|e| anyhow!("Can't get modified time: {}", e))
            .map(DateTime::<Utc>::from)
            .map(|d| d.with_nanosecond(0).unwrap_or_default())
            .map_err(|e| anyhow!("Can't convert to DateTime: {}", e))
    }
}

pub trait StringUtils {
    fn has_image_ext(&self) -> bool;
    fn has_archive_ext(&self) -> bool;
}

impl StringUtils for String {
    /// Check if the string has an image extension
    fn has_image_ext(&self) -> bool {
        self.split('.')
            .last()
            .map(|ext| SUPPORTED_IMAGE_FORMATS.contains(&ext))
            .unwrap_or(false)
    }

    /// Check if the string has an archive extension
    fn has_archive_ext(&self) -> bool {
        self.split('.')
            .last()
            .map(|ext| SUPPORTED_ARCHIVE_FORMATS.contains(&ext))
            .unwrap_or(false)
    }
}

pub trait WarnResultThenOk<T> {
    /// Same as [`Result::ok`] but log a warning if the result is an error
    fn okay(self, msg: impl Display) -> Option<T>;
}

impl<T, E: Display + Debug> WarnResultThenOk<T> for anyhow::Result<T, E> {
    fn okay(self, msg: impl Display) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!("{}: {:?}", msg, e);
                None
            }
        }
    }
}
