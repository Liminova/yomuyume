use std::{collections::VecDeque, path::PathBuf};

use anyhow::{anyhow, Result};

use crate::config::SUPPORTED_IMAGE_FORMATS;

use super::config::SUPPORTED_ARCHIVE_FORMATS;

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
    /// Only check if the [`PathBuf`] has an image extension.
    fn has_image_extension(&self) -> bool;

    /// Only check if the [`PathBuf`] has an archive extension.
    fn has_archive_extension(&self) -> bool;

    /// Scan a directory recursively for image files with respect
    /// to the provided [`nomedia_support`] feature flag.
    fn scan_dir_recursively_for_image(&self, nomedia_support: bool) -> Result<Vec<PathBuf>>;
}

impl PathBufUtils for PathBuf {
    fn has_image_extension(&self) -> bool {
        if let Some(extension) = self.extension() {
            return SUPPORTED_IMAGE_FORMATS
                .contains(&extension.to_string_lossy().to_string().as_ref());
        }
        false
    }

    fn has_archive_extension(&self) -> bool {
        if let Some(extension) = self.extension() {
            return SUPPORTED_ARCHIVE_FORMATS
                .contains(&extension.to_string_lossy().to_string().as_ref());
        }
        false
    }

    fn scan_dir_recursively_for_image(&self, nomedia_support: bool) -> Result<Vec<PathBuf>> {
        let mut files = vec![];
        let mut queue: VecDeque<PathBuf> = VecDeque::from([self.clone()]);

        while let Some(entry) = queue.pop_back() {
            if entry.is_file() {
                if nomedia_support && entry.join(".nomedia").exists() {
                    continue;
                }
                if entry.has_image_extension() {
                    files.push(entry);
                }
                continue;
            }
            queue.push_front(entry);
        }
        Ok(files)
    }
}

pub trait StringUtils {
    fn has_image_extension(&self) -> bool;
    fn has_archive_extension(&self) -> bool;
}

impl StringUtils for String {
    fn has_image_extension(&self) -> bool {
        if let Some(extension) = self.split('.').last() {
            return SUPPORTED_IMAGE_FORMATS.contains(&extension);
        }
        false
    }

    fn has_archive_extension(&self) -> bool {
        if let Some(extension) = self.split('.').last() {
            return SUPPORTED_ARCHIVE_FORMATS.contains(&extension);
        }
        false
    }
}
