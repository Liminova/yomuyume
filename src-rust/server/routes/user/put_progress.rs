use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};

use crate::{
    routes::errors::InternalErr,
    structs::ids::UserID,
    utils::{app_state::AppState, constants::USER_PROGRESS_PATH},
};

/// Set reading progress
#[utoipa::path(
    put,
    path = USER_PROGRESS_PATH,
    responses(
        (status = 200, description = "Set progress successfully"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("title_id" = i64, Query, description = "Title ID"),
        ("page_id" = i64, Query, description = "Page ID"),
        ("percent" = u8, Query, description = "Progress percent (0-100)"),
    ),
    security(("session-id" = [], "session-secret" = [])))
]
pub async fn put_progress(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path((title_id, page)): Path<(i64, i32)>,
) -> Result<Response, InternalErr> {
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
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}
