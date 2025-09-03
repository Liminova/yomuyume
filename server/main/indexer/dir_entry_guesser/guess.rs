use std::fs::DirEntry;
use tracing::warn;

use crate::{
    indexer::dir_entry_guesser::{PartialIndexedChapter, has_series_pattern::HasPatternOfSeries},
    utils::{
        archive_file::{ArchiveFile, ArchiveFileError, ItemInArchive, ItemsInArchiveUtils},
        pathbuf_utils::PathBufUtils,
        result_utils::ResultUtils,
    },
};

#[derive(Debug)]
pub enum DirEntryType {
    /// Contains **un**filtered list of `directories` in the category.
    CategoryDir(Vec<DirEntry>),
    /// Contains filtered list of `chapters` (directories and archives) in the series.
    SeriesDir(Vec<PartialIndexedChapter>),
    /// Contains **un**filtered list of `files and directories` in the directory.
    OneShotDir(Vec<DirEntry>),
    /// Contain **un**filtered list of `files` in the archive.
    OneShotArchiveFile(Vec<ItemInArchive>),
    Ignored,
}

#[derive(Debug, thiserror::Error)]
pub enum DirEntryTypeGuesserError {
    #[error("archive error: {0}")]
    ArchiveError(ArchiveFileError),
    #[error("read directory error: {0}")]
    ReadDir(std::io::Error),
}

pub trait DirEntryTypeGuesser {
    fn guess(
        &self,
        nomedia_support: bool,
        komga_oneshot_support: bool,
        komga_recycle_support: bool,
    ) -> Result<DirEntryType, DirEntryTypeGuesserError>;
}

impl DirEntryTypeGuesser for DirEntry {
    /// Guess the type of the [`DirEntry`] (represents both a file and a
    /// directory) while respecting the nomedia and komga feature flags.
    ///
    /// Check `docs/managing-library.md` for the decision tree diagram.
    fn guess(
        &self,
        nomedia_support: bool,
        komga_oneshot_support: bool,
        komga_recycle_support: bool,
    ) -> Result<DirEntryType, DirEntryTypeGuesserError> {
        let path = self.path();

        if path.is_file() {
            if !path.has_archive_ext() {
                return Ok(DirEntryType::Ignored);
            }
            let archive = (&path)
                .list_files_in_archive()
                .map_err(DirEntryTypeGuesserError::ArchiveError)?;
            if archive.is_empty()
                || archive.contains_nomedia(nomedia_support)
                || !archive.contains_image()
            {
                return Ok(DirEntryType::Ignored);
            }
            return Ok(DirEntryType::OneShotArchiveFile(archive));
        }

        if path.contains_nomedia_file(nomedia_support) {
            return Ok(DirEntryType::Ignored);
        }

        let items_in_dir: Vec<DirEntry> = path
            .read_dir()
            .map_err(DirEntryTypeGuesserError::ReadDir)?
            .filter_map(|e| {
                e.okay(|e| warn!("can't extract item from parent `{}`: {e:?}", path.display()))
            })
            .collect::<Vec<_>>();

        if items_in_dir.is_empty() {
            return Ok(DirEntryType::Ignored);
        }

        if path.contains_category_info_file() {
            return Ok(DirEntryType::CategoryDir(items_in_dir));
        }

        // only analyze the patterns to see if it's a series either when
        let not_force_oneshot = !path.has_oneshot_flag(komga_oneshot_support);
        let not_force_ignore = !path.has_recycle_flag(komga_recycle_support);
        if (!not_force_oneshot || !not_force_ignore)
            && let Some(chapter_infos) =
                items_in_dir.has_series_pattern(nomedia_support, komga_recycle_support)
        {
            return Ok(DirEntryType::SeriesDir(chapter_infos));
        }

        let contains_more_than_one_image_file = items_in_dir
            .iter()
            .filter(|entry| entry.path().has_image_ext())
            .count()
            > 1;

        if contains_more_than_one_image_file && !path.has_recycle_flag(komga_recycle_support) {
            return Ok(DirEntryType::OneShotDir(items_in_dir));
        }

        Ok(DirEntryType::CategoryDir(items_in_dir))
    }
}
