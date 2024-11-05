use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};

use crate::{types::UserID, AppError, AppState};

#[utoipa::path(put, path = "/api/user/progress/{title_id}/{page}", responses(
    (status = 200, description = "Set progress successfully"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn put_progress(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path((title_id, page)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    let result = sqlx::query!(
        "INSERT INTO progresses (user_id, title_id, last_read_at, page) VALUES ($1, $2, $3, $4)",
        user_id,
        title_id,
        chrono::Utc::now(),
        page as i32
    )
    .execute(&app_state.pool)
    .await;

    match result {
        Ok(_) => Ok((StatusCode::OK).into_response()),
        Err(sqlx::Error::Database(db_err))
            if db_err.constraint() == Some("progresses_title_id_fkey") =>
        {
            Ok((StatusCode::BAD_REQUEST, "invalid title_id").into_response())
        }
        Err(e) => Ok((StatusCode::INTERNAL_SERVER_ERROR, format!("{e:#}")).into_response()),
    }
}
