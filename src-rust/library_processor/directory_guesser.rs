use anyhow::{Context, Result};
use std::{collections::HashMap, fs::DirEntry, path::PathBuf};

use crate::{SUPPORTED_ARCHIVE_FORMATS, SUPPORTED_IMAGE_FORMATS};

type SubEntries = Vec<DirEntry>;
type ChapterPathAndNumber = HashMap<PathBuf, i32>;
type PagePaths = Vec<PathBuf>;

#[derive(Debug)]
pub enum DirEntryType {
    CategoryDir(SubEntries),
    SeriesDir(ChapterPathAndNumber),
    OneShotDir(PagePaths),
    OneShotArchiveFile,
    /// ALL sub-DirEntries are considered as one-shots.
    ForcedOneShotDir,
    Ignored,
}

/// Check README.md in this directory for visual explanation.
pub fn guess(
    entry: &DirEntry,
    nomedia_support: bool,
    komga_oneshot_support: bool,
    komga_recycle_support: bool,
) -> Result<DirEntryType> {
    let entry_path = entry.path();

    if komga_recycle_support && entry_path.to_string_lossy().contains("#recycle") {
        return Ok(DirEntryType::Ignored);
    }

    if entry
        .file_type()
        .context("can't get entry's file type")?
        .is_file()
    {
        if let Some(extension) = entry_path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
        {
            if SUPPORTED_ARCHIVE_FORMATS.contains(&extension.as_str()) {
                return Ok(DirEntryType::OneShotArchiveFile);
            }
        }
        return Ok(DirEntryType::Ignored);
    }

    if nomedia_support && entry_path.join(".nomedia").exists() {
        return Ok(DirEntryType::Ignored);
    }
    if komga_oneshot_support
        && entry_path.components().any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .ends_with("_oneshot")
        })
    {
        return Ok(DirEntryType::ForcedOneShotDir);
    }

    let sub_entries = entry_path
        .read_dir()
        .context("entry is category, but can't scan subdirs")?
        .try_fold(vec![], |mut subdirs, entry| {
            subdirs.push(entry.context("can't extract entry from parent")?);
            Ok::<_, anyhow::Error>(subdirs)
        })
        .context("entry is category, but can't build subdirs list")?;

    if entry_path.join("CategoryInfo.xml").exists() {
        return Ok(DirEntryType::CategoryDir(sub_entries));
    }

    let (subdirs_and_archives, images) = sub_entries
        .iter()
        .try_fold(
            (vec![], vec![]),
            |(mut subdirs_and_archives, mut images), entry| {
                let entry_path = entry.path();
                let entry_file_type = entry
                    .file_type()
                    .context(format!("can't get file type for {}", entry_path.display()))?;
                if entry_file_type.is_dir() {
                    subdirs_and_archives.push(entry_path);
                    return Ok::<_, anyhow::Error>((subdirs_and_archives, images));
                }
                if let Some(extension) = entry_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                {
                    if SUPPORTED_ARCHIVE_FORMATS.contains(&extension.as_str()) {
                        subdirs_and_archives.push(entry_path.clone());
                    }
                    if SUPPORTED_IMAGE_FORMATS.contains(&extension.as_str()) {
                        images.push(entry_path);
                    }
                }
                Ok::<_, anyhow::Error>((subdirs_and_archives, images))
            },
        )
        .context("can't build lists to check for series pattern")?;

    if let Some(result) = has_pattern_of_a_series(subdirs_and_archives) {
        return Ok(DirEntryType::SeriesDir(result));
    };

    if entry_path.join("ComicInfo.xml").exists() {
        return Ok(DirEntryType::OneShotDir(images));
    }

    if images.len() > 1 {
        return Ok(DirEntryType::OneShotDir(images));
    }

    return Ok(DirEntryType::CategoryDir(sub_entries));
}

/// If the items have a pattern of a series, return their
/// chapter numbers mapped respectively, None otherwise.
fn has_pattern_of_a_series(items: Vec<PathBuf>) -> Option<ChapterPathAndNumber> {
    let mut basename = "".to_string();
    let mut chapter_numbers: HashMap<PathBuf, i32> = HashMap::new();

    for item in items.iter() {
        let item_str = item.with_extension("");
        let item_str = match item_str.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return None,
        };

        let mut this_basename = String::with_capacity(item_str.len());
        let mut chapter_number = String::with_capacity(item_str.len());

        for ch in item_str.chars().rev() {
            let is_digit = ch.is_ascii_digit();
            if !is_digit && chapter_number.is_empty() {
                return None;
            }
            if is_digit && this_basename.is_empty() {
                chapter_number.push(ch);
                continue;
            }
            this_basename.push(ch.to_ascii_lowercase());
        }

        let this_basename = this_basename
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .rev()
            .collect::<String>();
        if !basename.is_empty() && this_basename != basename {
            return None;
        }
        basename = this_basename;
        chapter_numbers.insert(
            (*item).clone(),
            chapter_number
                .chars()
                .rev()
                .collect::<String>()
                .parse()
                .unwrap_or_default(),
        );
    }

    if chapter_numbers.is_empty() {
        return None;
    }

    Some(chapter_numbers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_pattern_of_a_series() {
        assert_eq!(
            has_pattern_of_a_series(vec![
                PathBuf::from("foo/bar/baz/123chapter_001.png"),
                PathBuf::from("foo/bar/baz/123chapter_002.png"),
                PathBuf::from("foo/bar/baz/123chapter_003.png"),
            ]),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/123chapter_001.png"), 1),
                (PathBuf::from("foo/bar/baz/123chapter_002.png"), 2),
                (PathBuf::from("foo/bar/baz/123chapter_003.png"), 3)
            ]))
        );
        assert_eq!(
            has_pattern_of_a_series(vec![
                PathBuf::from("foo/bar/baz/12chapter34_001.png"),
                PathBuf::from("foo/bar/baz/12chapter34_002.png"),
            ]),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/12chapter34_001.png"), 1),
                (PathBuf::from("foo/bar/baz/12chapter34_002.png"), 2)
            ]))
        );
        assert_eq!(
            has_pattern_of_a_series(vec![
                PathBuf::from("foo/bar/baz/chapter123_001.png"),
                PathBuf::from("foo/bar/baz/chapter_002.png"),
            ]),
            None
        );
        assert_eq!(
            has_pattern_of_a_series(vec![
                PathBuf::from("foo/bar/baz/Chapter_01.png"),
                PathBuf::from("foo/bar/baz/chapter0020.png"),
            ]),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/Chapter_01.png"), 1),
                (PathBuf::from("foo/bar/baz/chapter0020.png"), 20)
            ]))
        );
        assert_eq!(
            has_pattern_of_a_series(vec![
                PathBuf::from("foo/bar/baz/01.png"),
                PathBuf::from("foo/bar/baz/002.png"),
            ]),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/01.png"), 1),
                (PathBuf::from("foo/bar/baz/002.png"), 2)
            ]))
        );
    }
}
