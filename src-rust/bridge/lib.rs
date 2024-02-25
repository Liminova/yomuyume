pub mod routes;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Encode, Decode, Default)]
pub struct GenericResponseBody {
    pub message: String,
}

#[wasm_bindgen]
impl GenericResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> GenericResponseBody {
        bitcode::decode(payload).unwrap_or(GenericResponseBody {
            message: "Can't parse server response.".to_string(),
        })
    }
}
