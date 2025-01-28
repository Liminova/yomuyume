use std::sync::Arc;

use crate::AppState;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, IntoParams)]
pub struct StatusRequestBody {
    /// a test string to test your request body.
    pub echo: Option<String>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct StatusResponseBody {
    /// Current local server time.
    pub server_time: String,
    /// Current yomuyume version.
    pub version: String,
    /// Your test string.
    pub echo: Option<String>,
}

/// get server status
#[utoipa::path(get, path = "/api/utils/status", responses(
    (status = 200, description = "status check successful", body = StatusResponseBody)
))]
pub async fn get_status(State(app_state): State<Arc<AppState>>) -> Response {
    (
        StatusCode::OK,
        Json(StatusResponseBody {
            server_time: Local::now().to_string(),
            version: app_state.config.get_version(),
            echo: None,
        }),
    )
        .into_response()
}

/// post server status
#[utoipa::path(post, path = "/api/utils/status", responses(
    (status = 200, description = "status check successful", body = StatusResponseBody)
))]
pub async fn post_status(
    State(app_state): State<Arc<AppState>>,
    query: Json<StatusRequestBody>,
) -> Response {
    (
        StatusCode::OK,
        Json(StatusResponseBody {
            server_time: Local::now().to_string(),
            version: app_state.config.get_version(),
            echo: query.echo.clone(),
        }),
    )
        .into_response()
}
