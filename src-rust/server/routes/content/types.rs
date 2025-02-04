use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

#[skip_serializing_none]
#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct BasePageResponse {
    pub id: String,
    pub blurhash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub jxl: bool,
    pub description: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, ToSchema, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct BaseTitleResponse {
    pub id: String,
    pub title: Option<String>,
    pub author: Option<String>,

    pub category_id: Option<String>,
    pub description: Option<String>,
    pub release: Option<String>,

    pub cover_blurhash: Option<String>,
    /// Full cover width, you might want to clamp this down to a much,
    /// much smaller value (<32px) before decoding the blurhash
    pub cover_width: Option<i32>,
    /// Full cover height, you might want to clamp this down to a much,
    /// much smaller value (<32px) before decoding the blurhash
    pub cover_height: Option<i32>,
    pub cover_jxl: Option<bool>,

    pub date_updated: Option<String>,

    /// ID, Name
    pub tags: Option<Vec<(String, String)>>,
    /// Null if the value is 0
    pub favorites: Option<i64>,
    /// Null if the value is 0
    pub bookmarks: Option<i64>,

    pub is_favorite: bool,
    pub is_bookmark: bool,
    /// Null if the value is 0 or haven't read
    pub page_read: Option<i32>,
}
