use crate::{config, utils::constants::GET_STATUS_PATH};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, IntoParams)]
pub struct StatusRequest {
    /// a test string to test your request body.
    pub echo: Option<String>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Current local server time.
    pub server_time: String,
    /// Current yomuyume version.
    pub version: String,
    /// Your test string.
    pub echo: Option<String>,
}

/// Get server status
#[utoipa::path(
    get,
    path = GET_STATUS_PATH,
    responses(
        (status = 200, description = "Status check success", body = StatusResponse)
    ))
]
pub async fn get_status() -> Response {
    (
        StatusCode::OK,
        Json(StatusResponse {
            server_time: Local::now().to_string(),
            version: config::get_version(),
            echo: None,
        }),
    )
        .into_response()
}

/// Post server status
#[utoipa::path(
    post,
    path = GET_STATUS_PATH,
    responses(
        (status = 200, description = "Status check success", body = StatusResponse)
    ))
]
pub async fn post_status(query: Json<StatusRequest>) -> Response {
    (
        StatusCode::OK,
        Json(StatusResponse {
            server_time: Local::now().to_string(),
            version: config::get_version(),
            echo: query.echo.clone(),
        }),
    )
        .into_response()
}
