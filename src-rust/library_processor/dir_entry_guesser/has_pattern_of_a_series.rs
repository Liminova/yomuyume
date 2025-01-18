use std::fs::DirEntry;

use crate::{
    library_processor::dir_entry_guesser::{ScannedChapterInfo, ScannedChapterType},
    traits::pathbuf_utils::PathBufUtils,
    utils::{
        archive_file::{ArchiveFile, ItemsInArchiveUtils},
        macros::bail_if_empty,
    },
};

pub trait HasPatternOfSeries {
    fn has_pattern_of_a_series(
        &self,
        nomedia_support: bool,
        komga_recycle_support: bool,
    ) -> Option<Vec<ScannedChapterInfo>>;
}

impl HasPatternOfSeries for Vec<DirEntry> {
    fn has_pattern_of_a_series(
        &self,
        is_nomedia_enabled: bool,
        is_komga_recyle_enabled: bool,
    ) -> Option<Vec<ScannedChapterInfo>> {
        if self.len() < 2 {
            return None;
        }

        let are_we_chapters = self
            .iter()
            .map(|d| d.path())
            .filter(|p| !p.has_recycle_flag(is_komga_recyle_enabled))
            .filter_map(|p| match p.metadata() {
                Ok(m) => Some((p, m)),
                Err(e) => {
                    tracing::warn!("can't get metadata of `{}`: {e:?}", p.display());
                    None
                }
            })
            .filter(|(p, m)| {
                let is_file_but = m.is_file() && p.has_archive_ext();
                let is_dir_but = m.is_dir() && !p.contains_nomedia_file(is_nomedia_enabled);
                is_file_but || is_dir_but
            })
            .filter_map(|(p, m)| {
                if m.is_dir() {
                    return Some(ScannedChapterInfo {
                        volume: 0,
                        path: p,
                        dir_or_archive: ScannedChapterType::Directory,
                    });
                }

                if !p.has_archive_ext() {
                    return None;
                }

                p.list_files_in_archive()
                    .map(|files| (p.clone(), files))
                    .map_err(|e| {
                        tracing::warn!("can't list files in `{}`: {e:?}", p.display());
                    })
                    .ok()
                    .filter(|(_, files)| {
                        !files.is_empty()
                            || !files.contains_nomedia(is_nomedia_enabled)
                            || files.contains_image()
                    })
                    .map(|(p, files)| ScannedChapterInfo {
                        volume: 0,
                        path: p,
                        dir_or_archive: ScannedChapterType::Archive(files),
                    })
            })
            .collect::<Vec<_>>();

        bail_if_empty!(are_we_chapters, None);

        let mut basename = String::new();
        let mut chapters: Vec<ScannedChapterInfo> = Vec::with_capacity(are_we_chapters.len());

        for am_i_chapter in are_we_chapters {
            let path_last_component = am_i_chapter
                .path
                .with_extension("")
                .file_name()?
                .to_string_lossy()
                .to_string();

            let mut curr_basename_rev = String::with_capacity(path_last_component.len());
            let mut chapter_number = String::with_capacity(path_last_component.len());

            // iterate backwards
            for char in path_last_component.chars().rev() {
                let is_digit = char.is_ascii_digit();

                if !is_digit && chapter_number.is_empty() {
                    #[cfg(debug_assertions)]
                    tracing::debug!(
                        "returning None because `{:?}` doesn't have a chapter number",
                        am_i_chapter.path
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
                tracing::debug!(
                    "returning None because `{:?}` has a different basename than `{}`",
                    curr_basename_normalized,
                    basename
                );

                return None;
            }
            basename = curr_basename_normalized;

            chapters.push(ScannedChapterInfo {
                path: am_i_chapter.path,
                volume: chapter_number
                    .chars()
                    .rev()
                    .collect::<String>()
                    .parse()
                    .unwrap_or_default(),
                dir_or_archive: am_i_chapter.dir_or_archive,
            });
        }

        Some(chapters)
    }
}
