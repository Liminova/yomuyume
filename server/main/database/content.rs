use std::path::PathBuf;

use crate::utils::average_color::HexColor;
use chrono::{DateTime, Utc};
use redb::TableDefinition as TableDef;
use redb_macros::{RedbJsonValue, key_function};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// Identity Path is just file path relative to its parent
//
// - Page's path relative to its chapter
// - Chapter's path relative to its title
// - Title's path relative to library directory
//
// Thus we need to include their parents' Identity Path in the keys

pub type CategoryIdentityPath = String;
pub type TitleIdentityPath = String;
pub type ChapterIdentityPath = String;
pub type PageIdentityPath = String;

pub const CATEGORIES: TableDef<CategoryIdentityPath, Vec<TitleIdentityPath>> =
    TableDef::new("categories");

#[key_function]
pub type TitleKey = (Option<CategoryIdentityPath>, TitleIdentityPath);

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, RedbJsonValue)]
pub struct TitleInfo {
    pub chapters: Option<Vec<ChapterIdentityPath>>,

    /// NOTE: planned to use for archive title to decide wether to index or skip
    pub last_modified: Option<DateTime<Utc>>,
}

pub const TITLES: TableDef<TitleKey, TitleInfo> = TableDef::new("titles");

#[key_function]
pub type ChapterKey = (
    Option<CategoryIdentityPath>,
    TitleIdentityPath,
    Option<ChapterIdentityPath>,
);

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, RedbJsonValue)]
pub struct ChapterInfo {
    pub pages: Vec<PageIdentityPath>,
    pub fallback_vol_num: Option<u32>,
    pub cover: Option<(PageIdentityPath, HexColor)>,

    /// NOTE: same reason as above
    pub last_modified: Option<DateTime<Utc>>,
}

pub const CHAPTERS: TableDef<ChapterKey, ChapterInfo> = TableDef::new("chapters");

#[key_function]
type PageKey = (
    Option<CategoryIdentityPath>,
    TitleIdentityPath,
    Option<ChapterIdentityPath>,
    PageIdentityPath,
);

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, RedbJsonValue)]
pub struct PageInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub color: Option<HexColor>,
    pub size: Option<i64>,
    pub last_modified: Option<DateTime<Utc>>,

    /// Relative to title's path
    pub parent_path: PathBuf,
}

pub const PAGES: TableDef<PageKey, PageInfo> = TableDef::new("pages");
