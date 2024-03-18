use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;
use wasm_bindgen::prelude::*;

/* CATEGORIES */

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, Encode, Decode)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, ToSchema, Serialize, Deserialize, Encode, Decode)]
pub struct CategoriesResponseBody {
    pub data: Vec<CategoryResponse>,
}

#[wasm_bindgen]
impl CategoriesResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> Option<CategoriesResponseBody> {
        bitcode::decode(payload).ok()
    }
}

/* PAGES */

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, Encode, Decode)]
pub struct ResponsePage {
    pub id: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, Encode, Decode)]
pub struct ResponseCover {
    pub blurhash: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, ToSchema, Serialize, Deserialize, Encode, Decode)]
#[skip_serializing_none]
pub struct TitleResponseBody {
    pub category_id: String,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub cover: ResponseCover,
    pub tag_ids: Vec<u32>,
    pub pages: Vec<ResponsePage>,
    pub favorites: Option<i64>,
    pub bookmarks: Option<i64>,
    pub is_favorite: Option<bool>,
    pub is_bookmark: Option<bool>,
    pub page_read: Option<i64>,
    pub date_added: String,
    pub date_updated: String,
}

#[wasm_bindgen]
impl TitleResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> Option<TitleResponseBody> {
        bitcode::decode(payload).ok()
    }
}

/* FILTER */

#[derive(Debug, ToSchema, Serialize, Deserialize, Encode, Decode)]
pub struct FilterRequest {
    /// Keywords to search for (search in title, description, author, tags)
    pub keywords: Option<Vec<String>>,
    /// Categories to filter by
    pub category_ids: Option<Vec<String>>,
    /// Tags to filter by
    pub tag_ids: Option<Vec<i32>>,
    /// Maximum number of results to return
    pub limit: Option<u32>,

    pub is_reading: Option<bool>,
    pub is_finished: Option<bool>,
    pub is_bookmarked: Option<bool>,
    pub is_favorite: Option<bool>,

    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, Encode, Decode)]
#[skip_serializing_none]
pub struct FilterTitleResponseBody {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub category_id: String,
    pub release: Option<String>,
    pub favorite_count: Option<i64>,
    pub page_count: i64,
    pub page_read: Option<i64>,

    /// Cover
    pub blurhash: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, ToSchema, Serialize, Deserialize, Encode, Decode)]
pub struct FilterResponseBody {
    pub data: Vec<FilterTitleResponseBody>,
}

#[wasm_bindgen]
impl FilterResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> Option<FilterResponseBody> {
        bitcode::decode(payload).ok()
    }
}
