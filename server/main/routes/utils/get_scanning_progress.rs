use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::{app_state::AppState, constants::GET_SCANNING_PROGRESS_PATH};

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct ScanningProgressResponse {
    pub scanning_completed: bool,
    pub scanning_progress: f64,
}

/// Get library scanning progress
#[utoipa::path(
    get,
    path = GET_SCANNING_PROGRESS_PATH,
    responses(
        (status = 200, description = "Library scanning progress", body = ScanningProgressResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-id" = [], "session-secret" = [])))
]
pub async fn get_scanning_progress(State(app_state): State<Arc<AppState>>) -> Response {
    (
        StatusCode::OK,
        Json(ScanningProgressResponse {
            scanning_completed: *app_state.scanning_complete.read().await,
            scanning_progress: *app_state.scanning_progress.read().await,
        }),
    )
        .into_response()
}
