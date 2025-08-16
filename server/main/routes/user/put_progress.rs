use std::sync::Arc;

use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;

use crate::{
    routes::errors::InternalErr,
    structs::ids::UserID,
    utils::{app_state::AppState, constants::USER_PROGRESS_PATH},
};

#[derive(serde::Deserialize)]
pub struct PutProgressQuery {
    title_id: i64,
    chapter_id: i64,
    page_id: i64,
    percent: i16,
}

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
    Query(query): Query<PutProgressQuery>,
) -> Result<Response, InternalErr> {
    sqlx::query!(
        "INSERT INTO progresses (user_id, title_id, chapter_id, page_id, percent, last_read_at)
        VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (user_id, title_id) DO
        UPDATE
        SET chapter_id = EXCLUDED.chapter_id,
            page_id = EXCLUDED.page_id,
            percent = EXCLUDED.percent,
            last_read_at = EXCLUDED.last_read_at",
        user_id.as_ref(),
        query.title_id,
        query.chapter_id,
        query.page_id,
        query.percent,
        chrono::Utc::now()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}
