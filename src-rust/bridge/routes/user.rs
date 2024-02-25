use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeleteRequest {
    pub password: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModifyRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub new_password: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResetRequest {
    pub password: String,
}
