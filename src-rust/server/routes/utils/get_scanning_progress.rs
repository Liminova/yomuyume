use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{routes::ErrRsp, AppState};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScanningProgressResponseBody {
    scanning_completed: bool,
    scanning_progress: f64,
}

#[utoipa::path(get, path = "/api/utils/scanning_progress", responses(
    (status = 200, description = "", body = ScanningProgressResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
))]
pub async fn get_scanning_progress(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ErrRsp> {
    let scanning_complete = data.scanning_complete.lock().await;
    let scanning_progress = data.scanning_progress.lock().await;

    Ok((
        StatusCode::OK,
        Json(ScanningProgressResponseBody {
            scanning_completed: *scanning_complete,
            scanning_progress: *scanning_progress,
        }),
    ))
}
