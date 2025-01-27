use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::app_state::AppState;

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct ScanningProgressResponseBody {
    pub scanning_completed: bool,
    pub scanning_progress: f64,
}

/// get library scanning progress
#[utoipa::path(get, path = "/api/utils/scanning_progress", responses(
    (status = 200, description = "library scanning progress", body = ScanningProgressResponseBody),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_scanning_progress(State(app_state): State<Arc<AppState>>) -> Response {
    (
        StatusCode::OK,
        Json(ScanningProgressResponseBody {
            scanning_completed: *app_state.scanning_complete.lock().await,
            scanning_progress: *app_state.scanning_progress.lock().await,
        }),
    )
        .into_response()
}
