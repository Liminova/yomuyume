use std::sync::Arc;

use crate::{AppError, AppState};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};

#[utoipa::path(put, path = "/api/user/favorite/{id}", responses(
    (status = 200, description = "Add favorite successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn put_favorite(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<String>,
    Path(title_id): Path<String>,
) -> Result<Response, AppError> {
    let title_exists = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM titles WHERE id = $1) AS "exists!""#,
        &title_id
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't check if title exists")?
    .exists;
    if !title_exists {
        return Ok((StatusCode::BAD_REQUEST, "invalid title id").into_response());
    }

    sqlx::query!(
        r#"INSERT INTO favorites (title_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (title_id, user_id)
        DO NOTHING"#,
        &title_id,
        user_id.as_str()
    )
    .execute(&app_state.pool)
    .await
    .context("can't insert favorite")?;

    Ok((StatusCode::OK).into_response())
}

#[utoipa::path(put, path = "/user/favorite/{id}", responses(
    (status = 200, description = "Add bookmark successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn put_bookmark(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<String>,
    Path(title_id): Path<String>,
) -> Result<Response, AppError> {
    let title_exists = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM titles WHERE id = $1) AS "exists!""#,
        &title_id
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't check if title exists")?
    .exists;
    if !title_exists {
        return Ok((StatusCode::BAD_REQUEST, "invalid title id").into_response());
    }

    sqlx::query!(
        r#"INSERT INTO bookmarks (title_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (title_id, user_id)
        DO NOTHING"#,
        &title_id,
        &user_id
    )
    .execute(&app_state.pool)
    .await
    .context("can't insert bookmark")?;

    Ok((StatusCode::OK).into_response())
}

#[utoipa::path(delete, path = "/api/user/favorite/{id}", responses(
    (status = 200, description = "Delete favorite successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn delete_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user_id): Extension<String>,
    Path(title_id): Path<String>,
) -> Result<Response, AppError> {
    let title_exists = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM titles WHERE id = $1) AS "exists!""#,
        &title_id
    )
    .fetch_one(&data.pool)
    .await
    .context("can't check if title exists")?
    .exists;
    if !title_exists {
        return Ok((StatusCode::BAD_REQUEST, "invalid title id").into_response());
    }

    sqlx::query!(
        r#"DELETE FROM favorites WHERE title_id = $1 AND user_id = $2"#,
        &title_id,
        user_id.as_str()
    )
    .execute(&data.pool)
    .await
    .context("can't delete favorite")?;

    Ok((StatusCode::OK).into_response())
}

#[utoipa::path(delete, path = "/user/favorite/{id}", responses(
    (status = 200, description = "Delete bookmark successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn delete_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user_id): Extension<String>,
    Path(title_id): Path<String>,
) -> Result<Response, AppError> {
    let title_exists = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM titles WHERE id = $1) AS "exists!""#,
        &title_id
    )
    .fetch_one(&data.pool)
    .await
    .context("can't check if title exists")?
    .exists;
    if !title_exists {
        return Ok((StatusCode::BAD_REQUEST, "invalid title id").into_response());
    }

    sqlx::query!(
        r#"DELETE FROM bookmarks WHERE title_id = $1 AND user_id = $2"#,
        &title_id,
        &user_id
    )
    .execute(&data.pool)
    .await
    .context("can't delete bookmark")?;

    Ok((StatusCode::OK).into_response())
}
