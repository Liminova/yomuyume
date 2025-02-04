use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};

use crate::{routes::errors::InternalError, types::id::UserID, utils::app_state::AppState};

/// Set reading progress
#[utoipa::path(put, path = "/api/user/progress/{title_id}/{page}", responses(
    (status = 200, description = "Set progress successfully"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn put_progress(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path((title_id, page)): Path<(i64, i32)>,
) -> Result<Response, InternalError> {
    sqlx::query!(
        "INSERT INTO progresses (user_id, title_id, last_read_at, page)
        VALUES ($1, $2, $3, $4) ON CONFLICT (user_id, title_id) DO
        UPDATE
        SET last_read_at = EXCLUDED.last_read_at,
            page = EXCLUDED.page",
        user_id.as_ref(),
        title_id,
        chrono::Utc::now(),
        page
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        InternalError::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}
