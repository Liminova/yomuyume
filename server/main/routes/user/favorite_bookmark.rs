use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    routes::errors::InternalErr,
    structs::ids::UserID,
    utils::{
        app_state::AppState,
        constants::{BOOKMARK_PATH, FAVORITE_PATH},
    },
};

/// Favorite a title
#[utoipa::path(
    put,
    path = FAVORITE_PATH,
    responses(
        (status = 200, description = "Add favorite successful"),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(("session-secret" = [])))
]
pub async fn put_favorite(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, InternalErr> {
    sqlx::query!(
        "INSERT INTO favorites (id, title_id, user_id)
        VALUES ($1, $2, $3) ON CONFLICT (title_id, user_id) DO NOTHING",
        app_state.id_generator.snowflake().await.map_err(|e| {
            tracing::error!("{e}");
            InternalErr::Snowflake(e)
        })?,
        title_id,
        user_id.as_ref()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}

/// Bookmark a title
#[utoipa::path(
    put,
    path = BOOKMARK_PATH,
    responses(
        (status = 200, description = "Add bookmark successful"),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(("user-id" = [], "session-secret" = [])))
]
pub async fn put_bookmark(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, InternalErr> {
    sqlx::query!(
        "INSERT INTO bookmarks (id, title_id, user_id)
        VALUES ($1, $2, $3) ON CONFLICT (title_id, user_id) DO NOTHING",
        app_state.id_generator.snowflake().await.map_err(|e| {
            tracing::error!("{e}");
            InternalErr::Snowflake(e)
        })?,
        title_id,
        user_id.as_ref()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}

/// Un-favorite a title
#[utoipa::path(
    delete,
    path = FAVORITE_PATH,
    responses(
        (status = 200, description = "Delete favorite successful"),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(("user-id" = [], "session-secret" = [])))
]
pub async fn delete_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, InternalErr> {
    sqlx::query!(
        "DELETE FROM favorites
        WHERE title_id = $1
            AND user_id = $2",
        title_id,
        user_id.as_ref()
    )
    .execute(&data.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}

/// Un-bookmark a title
#[utoipa::path(
    delete,
    path = BOOKMARK_PATH,
    responses(
        (status = 200, description = "Delete bookmark successful"),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(("user-id" = [], "session-secret" = [])))
]
pub async fn delete_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Path(title_id): Path<i64>,
) -> Result<Response, InternalErr> {
    sqlx::query!(
        "DELETE FROM bookmarks
        WHERE title_id = $1
            AND user_id = $2",
        title_id,
        user_id.as_ref()
    )
    .execute(&data.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}
