use std::{collections::HashMap, fs::DirEntry, path::PathBuf};

use anyhow::{Context, Result};

use crate::{
    ArchiveFile, CATEGORY_INFO_FILENAME, SUPPORTED_ARCHIVE_FORMATS, SUPPORTED_IMAGE_FORMATS,
};

type SubEntries = Vec<DirEntry>;
type ChapterPathAndNumber = HashMap<PathBuf, i32>;
type PagePaths = Vec<PathBuf>;

#[derive(Debug)]
pub enum DirEntryType {
    CategoryDir(SubEntries),
    SeriesDir(ChapterPathAndNumber),
    OneShotDir(PagePaths),
    OneShotArchiveFile(ArchiveFile),
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
    /// Check docs/managing-library.md for the decision tree diagram.
    ///
    /// [`DirEntry`] in Rust represents both a file and a directory.
    fn guess(
        &self,
        nomedia_support: bool,
        komga_oneshot_support: bool,
        komga_recycle_support: bool,
    ) -> Result<DirEntryType> {
        let entry_path = self.path();

        match self.is_supported_archive()? {
            DirEntryBasicType::File => return Ok(DirEntryType::Ignored),
            DirEntryBasicType::SupportedArchive => {
                if entry_path.has_recycle_flag(komga_recycle_support) {
                    return Ok(DirEntryType::Ignored);
                }
                return Ok(DirEntryType::OneShotArchiveFile(
                    ArchiveFile::from_unchecked(entry_path),
                ));
            }
            DirEntryBasicType::Directory => {}
        }

        if entry_path.contains_nomedia_file() {
            return Ok(DirEntryType::Ignored);
        }

        let sub_dir_entries = entry_path
            .get_sub_dir_entries(nomedia_support)
            .context("can't get subdirs of the entry")?;

        if entry_path.contains_category_info() {
            return Ok(DirEntryType::CategoryDir(sub_dir_entries));
        }

        let (subdirs_and_archives, images) = sub_dir_entries
            .organize_to_entries_and_images()
            .context("can't organize subdirs to check for series pattern")?;

        if !entry_path.has_oneshot_flag(komga_oneshot_support) {
            if let Some(chap_path_and_number) = subdirs_and_archives.has_pattern_of_a_series() {
                if entry_path.has_recycle_flag(komga_recycle_support) {
                    return Ok(DirEntryType::Ignored);
                }
                return Ok(DirEntryType::SeriesDir(chap_path_and_number));
            }
        }

        if images.len() > 1 {
            if entry_path.has_recycle_flag(komga_recycle_support) {
                return Ok(DirEntryType::Ignored);
            }
            return Ok(DirEntryType::OneShotDir(images));
        }

        Ok(DirEntryType::CategoryDir(sub_dir_entries))
    }
}

#[derive(Debug)]
enum DirEntryBasicType {
    File,
    Directory,
    SupportedArchive,
}

trait IsSupportedArchive {
    fn is_supported_archive(&self) -> Result<DirEntryBasicType>;
}

impl IsSupportedArchive for DirEntry {
    /// Check if the [`DirEntry`] is a supported archive file.
    fn is_supported_archive(&self) -> Result<DirEntryBasicType> {
        if self
            .file_type()
            .context("can't get entry's file type")?
            .is_file()
        {
            if let Some(extension) = self
                .path()
                .extension()
                .map(|e| e.to_string_lossy().to_string())
            {
                if SUPPORTED_ARCHIVE_FORMATS.contains(&extension.as_str()) {
                    return Ok(DirEntryBasicType::SupportedArchive);
                }
            }
            return Ok(DirEntryBasicType::File);
        }
        Ok(DirEntryBasicType::Directory)
    }
}

trait HasKomgaFlag {
    fn has_flag(&self, flag: &str) -> bool;
    fn has_recycle_flag(&self, feature_enabled: bool) -> bool;
    fn has_oneshot_flag(&self, feature_enabled: bool) -> bool;
}

impl HasKomgaFlag for PathBuf {
    fn has_flag(&self, flag: &str) -> bool {
        if let Some(prefix_in_its_name) = self
            .file_name()
            .map(|p| p.to_string_lossy().starts_with(flag))
        {
            if prefix_in_its_name {
                return true;
            }
        }

        let somewhere_in_parent = self.components().any(|component| {
            let c = component.as_os_str().to_string_lossy();
            !c.starts_with(flag) && c.contains(flag)
        });

        somewhere_in_parent
    }

    /// Check if any of the path's components contains Komga's `_oneshot`
    fn has_oneshot_flag(&self, feature_enabled: bool) -> bool {
        if !feature_enabled {
            return false;
        }
        self.has_flag("_oneshot")
    }

    /// Check if any of the path's components contains Komga's `#recycle`
    fn has_recycle_flag(&self, feature_enabled: bool) -> bool {
        if !feature_enabled {
            return false;
        }
        self.has_flag("#recycle")
    }
}

trait HasPatternOfSeries {
    /// If the items have a pattern of a series, return their
    /// chapter numbers mapped respectively, None otherwise.
    fn has_pattern_of_a_series(&self) -> Option<ChapterPathAndNumber>;
}

impl HasPatternOfSeries for Vec<PathBuf> {
    /// If the items have a pattern of a series, return their
    /// chapter numbers mapped respectively, None otherwise.
    fn has_pattern_of_a_series(&self) -> Option<ChapterPathAndNumber> {
        if self.len() < 2 {
            return None;
        }

        let mut basename = "".to_string();
        let mut chapter_numbers: HashMap<PathBuf, i32> = HashMap::new();

        for item in self.iter() {
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
}

trait ContainsFile {
    fn contains_category_info(&self) -> bool;
    fn contains_comic_info(&self) -> bool;
    fn contains_nomedia_file(&self) -> bool;
}

impl ContainsFile for PathBuf {
    /// Check if the directory contains CategoryInfo.xml
    ///
    /// This method DOES NOT check if the given PathBuf is a directory.
    fn contains_category_info(&self) -> bool {
        self.join(CATEGORY_INFO_FILENAME).exists()
    }

    /// Check if the directory contains ComicInfo.xml
    ///
    /// This method DOES NOT check if the given PathBuf is a directory.
    fn contains_comic_info(&self) -> bool {
        self.join(COMICINFO_FILENAME).exists()
    }

    /// Check if the directory contains .nomedia
    ///
    /// This method DOES NOT check if the given PathBuf is a directory.
    fn contains_nomedia_file(&self) -> bool {
        self.join(".nomedia").exists()
    }
}

trait Organize {
    fn organize_to_entries_and_images(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>)>;
}

impl Organize for Vec<DirEntry> {
    /// Organize a [`Vec<DirEntry>`] to 2 [`Vec<PathBuf>`]
    /// - One containing directories and archives
    /// - One containing images
    fn organize_to_entries_and_images(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        self.iter()
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
            .context("can't build lists to check for series pattern")
    }
}

trait GetSubDirEntries {
    fn get_sub_dir_entries(&self, nomedia_support: bool) -> Result<Vec<DirEntry>>;
}

impl GetSubDirEntries for PathBuf {
    fn get_sub_dir_entries(&self, nomedia_support: bool) -> Result<Vec<DirEntry>> {
        self.read_dir()
            .context(format!("can't read path {} as a directory", self.display()))?
            .try_fold(vec![], |mut subdirs, entry| {
                let entry = entry.context("can't extract entry from parent")?;
                let nomedia_exists = entry.path().join(".nomedia").exists();
                if nomedia_support && nomedia_exists {
                    return Ok::<_, anyhow::Error>(subdirs);
                }
                subdirs.push(entry);
                Ok::<_, anyhow::Error>(subdirs)
            })
            .context("can't build subdirs list")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_pattern_of_a_series() {
        assert_eq!(
            vec![
                PathBuf::from("foo/bar/baz/123chapter_001.png"),
                PathBuf::from("foo/bar/baz/123chapter_002.png"),
                PathBuf::from("foo/bar/baz/123chapter_003.png"),
            ]
            .has_pattern_of_a_series(),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/123chapter_001.png"), 1),
                (PathBuf::from("foo/bar/baz/123chapter_002.png"), 2),
                (PathBuf::from("foo/bar/baz/123chapter_003.png"), 3)
            ]))
        );
        assert_eq!(
            vec![
                PathBuf::from("foo/bar/baz/12chapter34_001.png"),
                PathBuf::from("foo/bar/baz/12chapter34_002.png"),
            ]
            .has_pattern_of_a_series(),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/12chapter34_001.png"), 1),
                (PathBuf::from("foo/bar/baz/12chapter34_002.png"), 2)
            ]))
        );
        assert_eq!(
            vec![
                PathBuf::from("foo/bar/baz/chapter123_001.png"),
                PathBuf::from("foo/bar/baz/chapter_002.png"),
            ]
            .has_pattern_of_a_series(),
            None
        );
        assert_eq!(
            vec![
                PathBuf::from("foo/bar/baz/Chapter_01.png"),
                PathBuf::from("foo/bar/baz/chapter0020.png"),
            ]
            .has_pattern_of_a_series(),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/Chapter_01.png"), 1),
                (PathBuf::from("foo/bar/baz/chapter0020.png"), 20)
            ]))
        );
        assert_eq!(
            vec![
                PathBuf::from("foo/bar/baz/01.png"),
                PathBuf::from("foo/bar/baz/002.png"),
            ]
            .has_pattern_of_a_series(),
            Some(HashMap::from([
                (PathBuf::from("foo/bar/baz/01.png"), 1),
                (PathBuf::from("foo/bar/baz/002.png"), 2)
            ]))
        );
    }
}
