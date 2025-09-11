use std::sync::Arc;

use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use chrono::Utc;
use tracing::error;

use crate::{
    AppState,
    routes::{UserIDExtension, errors::InternalError},
    utils::constants::PUT_READ_PROGRESS_PATH,
};

#[derive(serde::Deserialize)]
pub struct PutProgressQuery {
    title_id: String,
    chapter_id: String,
    page_number: u32,
}

/// Set reading progress
#[utoipa::path(
    put,
    path = PUT_READ_PROGRESS_PATH,
    responses(
        (status = 200, description = "Set progress successfully"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("title_id" = String, Query, description = "Title ID"),
        ("chapter_id" = String, Query, description = "Chapter ID"),
        ("chapter_id" = u32, Query, description = "Page number"),
    ),
    security(("session-secret" = []))
)]
pub async fn put_progress(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
    Query(query): Query<PutProgressQuery>,
) -> Result<Response, InternalError> {
    let Some(user_id) = user_id.0 else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let now = Utc::now().naive_utc();

    sqlx::query!(
        "INSERT INTO reading_progress (user_id, title_id, chapter_id, page_number, updated_at)
        VALUES (?, ?, ?, ?, ?) ON CONFLICT (user_id, title_id) DO
        UPDATE
        SET chapter_id = EXCLUDED.chapter_id,
            page_number = EXCLUDED.page_number,
            updated_at = EXCLUDED.updated_at",
        user_id,
        query.title_id,
        query.chapter_id,
        query.page_number,
        now
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't upsert reading progress: {e:?}"))?;

    Ok(StatusCode::OK.into_response())
}
