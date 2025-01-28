use anyhow::{Context, Result};
use std::fs::DirEntry;
use tracing::warn;

use crate::{
    library_processor::dir_entry_guesser::{
        has_pattern_of_a_series::HasPatternOfSeries, ScannedChapterInfo,
    },
    traits::{do_something_and_ok::DoSomethingAndOk, pathbuf_utils::PathBufUtils},
    utils::{
        archive_file::{ArchiveFile, ItemInArchive, ItemsInArchiveUtils},
        macros::bail_if_empty,
    },
};

#[derive(Debug)]
pub enum DirEntryType {
    /// Contains **un**filtered list of `directories` in the category.
    CategoryDir(Vec<DirEntry>),
    /// Contains filtered list of `chapters` (directories and archives) in the series.
    SeriesDir(Vec<ScannedChapterInfo>),
    /// Contains **un**filtered list of `files and directories` in the directory.
    OneShotDir(Vec<DirEntry>),
    /// Contain **un**filtered list of `files` in the archive.
    OneShotArchiveFile(Vec<ItemInArchive>),
    Ignored,
}

pub trait DirEntryTypeGuesser {
    fn guess(
        &self,
        nomedia_support: bool,
        komga_oneshot_support: bool,
        komga_recycle_support: bool,
    ) -> Result<DirEntryType>;
}

impl DirEntryTypeGuesser for DirEntry {
    /// Guess the type of the [`DirEntry`] (in Rust represents both a file and a
    /// directory) while respecting the nomedia and komga flags.
    ///
    /// Check `docs/managing-library.md` for the decision tree diagram.
    fn guess(
        &self,
        nomedia_support: bool,
        komga_oneshot_support: bool,
        komga_recycle_support: bool,
    ) -> Result<DirEntryType> {
        let path = self.path();
        let path_str = path.to_string_lossy().to_string();

        if path.is_file() {
            if !path.has_archive_ext() {
                return Ok(DirEntryType::Ignored);
            }
            let archive = path
                .list_files_in_archive()
                .context("can't list files in archive")?;
            if archive.is_empty()
                || archive.contains_nomedia(nomedia_support)
                || !archive.contains_image()
            {
                return Ok(DirEntryType::Ignored);
            }
            return Ok(DirEntryType::OneShotArchiveFile(archive));
        }

        let dir = path;
        if dir.contains_nomedia_file(nomedia_support) {
            return Ok(DirEntryType::Ignored);
        }

        let items_in_dir = dir
            .read_dir()
            .context(format!("can't read `{path_str}` as a directory"))?
            .filter_map(|e| e.okay(|e| warn!("can't extract item from parent `{path_str}`: {e:?}")))
            .collect::<Vec<_>>();

        bail_if_empty!(items_in_dir, Ok(DirEntryType::Ignored));

        if dir.contains_category_info_file() {
            return Ok(DirEntryType::CategoryDir(items_in_dir));
        }

        // only analyze the patterns to see if it's a series either when
        // - i'm not forced to be a one-shot
        // - i'm not forced to be recycled
        if !dir.has_oneshot_flag(komga_oneshot_support)
            || !dir.has_recycle_flag(komga_recycle_support)
        {
            if let Some(chapter_infos) =
                items_in_dir.has_pattern_of_a_series(nomedia_support, komga_recycle_support)
            {
                return Ok(DirEntryType::SeriesDir(chapter_infos));
            }
        }

        let contains_more_than_one_image_file = items_in_dir
            .iter()
            .filter(|entry| entry.path().has_image_ext())
            .count()
            > 1;

        if contains_more_than_one_image_file && !dir.has_recycle_flag(komga_recycle_support) {
            return Ok(DirEntryType::OneShotDir(items_in_dir));
        }

        Ok(DirEntryType::CategoryDir(items_in_dir))
    }
}
