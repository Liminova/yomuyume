use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use wasm_bindgen::prelude::*;

/* LOGIN */

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Serialize, Deserialize, Encode, Decode, ToSchema)]
pub struct LoginResponseBody {
    pub token: String,
}

#[wasm_bindgen]
impl LoginResponseBody {
    pub fn from_bitcode(payload: &[u8]) -> Option<LoginResponseBody> {
        bitcode::decode(payload).ok()
    }
}

/* REGISTER */

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}
