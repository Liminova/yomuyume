use std::sync::Arc;

pub use bridge::routes::utils::ScanningProgressResponseBody;

use crate::{
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{extract::State, http::HeaderMap, response::IntoResponse};

#[utoipa::path(get, path = "/api/utils/scanning_progress", responses(
    (status = 200, description = "", body = ScanningProgressResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
))]
pub async fn get_scanning_progress(
    State(data): State<Arc<AppState>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let scanning_complete = data.scanning_complete.lock().await;
    let scanning_progress = data.scanning_progress.lock().await;

    Ok(builder.success(ScanningProgressResponseBody {
        scanning_completed: *scanning_complete,
        scanning_progress: *scanning_progress,
    }))
}
