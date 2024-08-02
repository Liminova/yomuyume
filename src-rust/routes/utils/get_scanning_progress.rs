use std::sync::Arc;

use crate::{AppError, AppState};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScanningProgressResponseBody {
    pub scanning_completed: bool,
    pub scanning_progress: f64,
}

#[utoipa::path(get, path = "/api/utils/scanning_progress", responses(
    (status = 200, description = "", body = ScanningProgressResponseBody),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn get_scanning_progress(
    State(data): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let scanning_complete = data.scanning_complete.lock().await;
    let scanning_progress = data.scanning_progress.lock().await;

    Ok((
        StatusCode::OK,
        Json(ScanningProgressResponseBody {
            scanning_completed: *scanning_complete,
            scanning_progress: *scanning_progress,
        }),
    )
        .into_response())
}
