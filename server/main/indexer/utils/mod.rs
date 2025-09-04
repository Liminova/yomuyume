pub mod comic_info_to_tantivy;
pub mod find_chapter_cover;
pub mod index_archive_chap_pages;
pub mod index_directory_chap_pages;

use chrono::{DateTime, Utc};

use crate::{database::content::PageIdentityPath, utils::average_color::HexColor};

#[derive(Debug)]
pub struct PageToUpsert {
    pub identity_path: PageIdentityPath,

    pub width: Option<u32>,
    pub height: Option<u32>,
    pub color: Option<HexColor>,
    pub size: Option<i64>,
    pub last_modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub struct IndexedChapterPages {
    pub upsert: Vec<PageToUpsert>,
    pub delete: Vec<PageIdentityPath>,
}
