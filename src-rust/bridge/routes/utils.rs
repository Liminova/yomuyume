use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use wasm_bindgen::prelude::*;
/* Scanning Progress */

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct ScanningProgressResponseBody {
    pub scanning_completed: bool,
    pub scanning_progress: f64,
}

#[wasm_bindgen]
impl ScanningProgressResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> Option<ScanningProgressResponseBody> {
        bitcode::decode(payload).ok()
    }
}

/* TagsMap */

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TagResponseBody {
    pub id: u32,
    pub name: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TagsMapResponseBody {
    pub data: Vec<TagResponseBody>,
}

#[wasm_bindgen]
impl TagsMapResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> Option<TagsMapResponseBody> {
        bitcode::decode(payload).ok()
    }
}

/* Status */

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema, Encode, Decode)]
pub struct StatusRequest {
    ///  A test string to test your request body.
    pub echo: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, ToSchema, Deserialize, Serialize, Encode, Decode)]
pub struct StatusResponseBody {
    /// Current local server time.
    pub server_time: String,
    /// Current yomuyume version.
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Your test string.
    pub echo: Option<String>,
}

#[wasm_bindgen]
impl StatusResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> Option<StatusResponseBody> {
        bitcode::decode(payload).ok()
    }
}
