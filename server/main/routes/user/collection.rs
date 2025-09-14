use std::sync::Arc;

use crate::{
    AppState,
    routes::{UserIDExtension, errors::InternalError},
    utils::{
        constants::{DELETE_COLLECTION_PATH, PUT_COLLECTION_PATH, TITLE_IN_COLLECTION_PATH},
        nanoid::nanoid,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

/// Create a new collection for the user.
#[utoipa::path(
    put,
    path = PUT_COLLECTION_PATH,
    responses(
        (status = 200, description = "Collection created successfully", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-secret" = []))
)]
pub async fn create_collection(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
    Path(name): Path<String>,
) -> Result<Response, InternalError> {
    let Some(user_id) = user_id.0 else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let id = nanoid();
    sqlx::query!(
        "INSERT INTO collections (id, user_id, name) VALUES (?, ?, ?)",
        id,
        user_id,
        name
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't create collection: {e:?}"))?;

    Ok((StatusCode::OK, id).into_response())
}

/// Put a title into a collection.
#[utoipa::path(
    put,
    path = TITLE_IN_COLLECTION_PATH,
    responses(
        (status = 200, description = "Title added to collection successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-secret" = []))
)]
pub async fn put_title_in_collection(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
    Path((collection_id, title_id)): Path<(String, String)>,
) -> Result<Response, InternalError> {
    if user_id.0.is_none() {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    sqlx::query!(
        "INSERT INTO collection_titles (collection_id, title_id) VALUES (?, ?)",
        collection_id,
        title_id
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't add title to collection: {e:?}"))?;

    Ok(StatusCode::OK.into_response())
}

/// Remove a title from a collection.
#[utoipa::path(
    delete,
    path = TITLE_IN_COLLECTION_PATH,
    responses(
        (status = 200, description = "Title removed from collection successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-secret" = []))
)]
pub async fn delete_title_from_collection(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
    Path((collection_id, title_id)): Path<(String, String)>,
) -> Result<Response, InternalError> {
    if user_id.0.is_none() {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    sqlx::query!(
        "DELETE FROM collection_titles WHERE collection_id = ? AND title_id = ?",
        collection_id,
        title_id
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't remove title from collection: {e:?}"))?;

    Ok(StatusCode::OK.into_response())
}

/// Delete a collection
#[utoipa::path(
    delete,
    path = DELETE_COLLECTION_PATH,
    responses(
        (status = 200, description = "Collection deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-secret" = []))
)]
pub async fn delete_collection(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
    Path(collection_id): Path<String>,
) -> Result<Response, InternalError> {
    if user_id.0.is_none() {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    sqlx::query!("DELETE FROM collections WHERE id = ?", collection_id)
        .execute(&app_state.pool)
        .await
        .inspect_err(|e| error!("can't remove collection: {e:?}"))?;

    Ok(StatusCode::OK.into_response())
}
