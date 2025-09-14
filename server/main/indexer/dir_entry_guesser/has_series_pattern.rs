use std::fs::DirEntry;

use tracing::{debug, warn};

use crate::{
    indexer::dir_entry_guesser::{IndexedChapterKind, PartialIndexedChapter},
    utils::{
        absolute_path::{ToAbsolute, VecAbsolutePathUtils},
        archive_file::{ArchiveFile, ItemsInArchiveUtils},
        pathbuf_utils::PathBufUtils,
        result_utils::ResultUtils,
    },
};

pub trait HasPatternOfSeries {
    fn has_series_pattern(&self, nomedia_support: bool) -> Option<Vec<PartialIndexedChapter>>;
}

impl HasPatternOfSeries for Vec<DirEntry> {
    fn has_series_pattern(&self, nomedia_support: bool) -> Option<Vec<PartialIndexedChapter>> {
        if self.len() < 2 {
            return None;
        }

        let mut partial_indexed_chapters: Vec<PartialIndexedChapter> = Vec::new();
        for entry in self {
            let path = entry.path();
            let Some(absolute_path) = path.to_absolute(None).okay(|e| {
                warn!("can't convert `{}` to absolute path: {e}", path.display());
            }) else {
                continue;
            };

            if path.is_dir() {
                if path.contains_nomedia_file(nomedia_support) {
                    continue;
                }
                let Some(items_in_dir) = absolute_path.list_items_in_directory().okay(|e| {
                    warn!("can't list items in `{}`: {e}", path.display());
                }) else {
                    continue;
                };
                if items_in_dir.is_empty() || !items_in_dir.contains_image() {
                    continue;
                }

                partial_indexed_chapters.push(PartialIndexedChapter {
                    fallback_vol_num: 0,
                    path: absolute_path,
                    kind: IndexedChapterKind::Directory(items_in_dir),
                });
                continue;
            }

            if !path.has_archive_ext() {
                continue;
            }

            let Some(files_in_archive) = path.list_files_in_archive().okay(|e| {
                warn!("can't list files in `{}`: {e}", path.display());
            }) else {
                continue;
            };

            if files_in_archive.is_empty()
                || files_in_archive.contains_nomedia(nomedia_support)
                || !files_in_archive.contains_image()
            {
                continue;
            }

            partial_indexed_chapters.push(PartialIndexedChapter {
                fallback_vol_num: 0,
                path: absolute_path,
                kind: IndexedChapterKind::Archive(files_in_archive),
            });
        }

        if partial_indexed_chapters.is_empty() {
            return None;
        }

        let mut basename = String::new();

        for partial_indexed_chapter in &mut partial_indexed_chapters {
            let title = partial_indexed_chapter
                .path
                .as_ref()
                .with_extension("")
                .file_name()?
                .to_string_lossy()
                .to_string();

            let mut curr_basename_rev = String::with_capacity(title.len());
            let mut chapter_number = String::with_capacity(title.len());

            // iterate backwards
            for char in title.chars().rev() {
                let is_digit = char.is_ascii_digit();

                if !is_digit && chapter_number.is_empty() {
                    #[cfg(debug_assertions)]
                    debug!(
                        "returning None because `{:?}` doesn't have a chapter number",
                        partial_indexed_chapter.path
                    );

                    return None;
                }

                // treat as a part of chapter number if haven't encountered
                // the basename yet (aka the basename still empty)
                if is_digit && curr_basename_rev.is_empty() {
                    chapter_number.push(char);
                    continue;
                }

                curr_basename_rev.push(char.to_ascii_lowercase());
            }

            // reverse & normalize the basename
            let curr_basename_normalized = curr_basename_rev
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .rev()
                .collect::<String>();

            // break immediately if there exists another basename
            if !basename.is_empty() && curr_basename_normalized != basename {
                #[cfg(debug_assertions)]
                debug!(
                    "returning None because `{:?}` has a different basename than `{}`",
                    curr_basename_normalized, basename
                );

                return None;
            }
            basename = curr_basename_normalized;

            partial_indexed_chapter.fallback_vol_num = chapter_number
                .chars()
                .rev()
                .collect::<String>()
                .parse()
                .unwrap_or_default();
        }

        Some(partial_indexed_chapters)
    }
}
