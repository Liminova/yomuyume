mod guess;
mod has_pattern_of_a_series;
mod tests;

use std::path::PathBuf;

use crate::archive_file::ItemInArchive;
pub use guess::{DirEntryType, DirEntryTypeGuesser};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ChapterDirOrArchive {
    Dir,
    Archive(Vec<ItemInArchive>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChapterInfo {
    /// Use as chapter's number unless overriden by its ComicInfo.xml.
    pub number: i32,
    /// Path to the chapter's directory or archive file.
    pub path: PathBuf,
    pub dir_or_archive: ChapterDirOrArchive,
}
