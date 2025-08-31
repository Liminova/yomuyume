mod guess;
mod has_series_pattern;
mod tests;

use crate::utils::{absolute_path::AbsolutePath, archive_file::ItemInArchive};
pub use guess::{DirEntryType, DirEntryTypeGuesser};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IndexedChapterKind {
    Directory,
    Archive(Vec<ItemInArchive>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PartialIndexedChapter {
    pub fallback_vol_num: u32,
    pub path: AbsolutePath,
    pub kind: IndexedChapterKind,
}
