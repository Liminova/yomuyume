use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};

use crate::{
    types::UserID,
    utils::{app_error::AppError, app_state::AppState},
};

/// add favorite
///
/// add a title to the user's favorites
#[utoipa::path(put, path = "/api/user/favorite/{title_id}", responses(
    (status = 200, description = "add favorite successful"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn put_favorite(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, AppError> {
    sqlx::query!(
        "INSERT INTO favorites (id, title_id, user_id) VALUES ($1, $2, $3)
        ON CONFLICT (title_id, user_id) DO NOTHING",
        app_state
            .id_generator
            .snowflake()
            .await
            .context("can't generate id")?,
        title_id,
        user_id
    )
    .execute(&app_state.pool)
    .await
    .context("can't upsert favorite")?;

    Ok((StatusCode::OK).into_response())
}

/// add bookmark
///
/// add a title to the user's bookmarks
#[utoipa::path(put, path = "/user/bookmark/{title_id}", responses(
    (status = 200, description = "add bookmark successful"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn put_bookmark(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, AppError> {
    sqlx::query!(
        "INSERT INTO bookmarks (id, title_id, user_id) VALUES ($1, $2, $3)
        ON CONFLICT (title_id, user_id) DO NOTHING",
        app_state
            .id_generator
            .snowflake()
            .await
            .context("can't generate id")?,
        title_id,
        &user_id
    )
    .execute(&app_state.pool)
    .await
    .context("can't upsert bookmark")?;

    Ok((StatusCode::OK).into_response())
}

/// delete favorite
///
/// delete a title from the user's favorites
#[utoipa::path(delete, path = "/api/user/favorite/{title_id}", responses(
    (status = 200, description = "delete favorite successful"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn delete_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, AppError> {
    sqlx::query!(
        "DELETE FROM favorites WHERE title_id = $1 AND user_id = $2",
        title_id,
        user_id
    )
    .execute(&data.pool)
    .await
    .context("can't delete favorite")?;

    Ok((StatusCode::OK).into_response())
}

/// delete bookmark
///
/// delete a title from the user's bookmarks
#[utoipa::path(delete, path = "/user/bookmark/{title_id}", responses(
    (status = 200, description = "delete bookmark successful"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn delete_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, AppError> {
    sqlx::query!(
        "DELETE FROM bookmarks WHERE title_id = $1 AND user_id = $2",
        title_id,
        &user_id
    )
    .execute(&data.pool)
    .await
    .context("can't delete bookmark")?;

    Ok((StatusCode::OK).into_response())
}
