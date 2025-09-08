pub mod comic_info_to_tantivy;
pub mod find_chapter_cover;
pub mod index_archive_chap_pages;
pub mod index_directory_chap_pages;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::utils::average_color::HexColor;

#[derive(Debug)]
pub struct PageInDB {
    pub id: String,
    pub path: String,
    pub last_modified: Option<DateTime<Utc>>,
    pub avg_color: Option<HexColor>,
}

#[derive(Debug)]
pub struct PageToUpsert {
    pub id: String,
    pub path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub avg_hex_color: Option<String>,
    pub size: Option<i64>,
    pub last_modified: Option<NaiveDateTime>,
}

type PageID = String;

#[derive(Debug, Default)]
pub struct IndexedChapterPages {
    pub upsert: Vec<PageToUpsert>,
    pub delete: Vec<PageID>,
}
