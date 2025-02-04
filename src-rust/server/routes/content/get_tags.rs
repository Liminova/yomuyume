use std::sync::Arc;

use crate::{
    routes::errors::InternalError,
    utils::{app_state::AppState, macros::bail_if_empty},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct InnerTagResponse {
    pub id: String,
    pub name: String,
}

type TagsResponse = Vec<InnerTagResponse>;

/// Get all tags
#[utoipa::path(get, path = "/api/content/tags", responses(
    (status = 200, description = "Get tags success", body = TagsResponse),
    (status = 204, description = "No tag found"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_tags(State(app_state): State<Arc<AppState>>) -> Result<Response, InternalError> {
    let data = sqlx::query!("SELECT id, name FROM tags")
        .fetch_all(&app_state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e:?}");
            InternalError::DB(e)
        })?
        .into_iter()
        .map(|record| InnerTagResponse {
            id: record.id.to_string(),
            name: record.name,
        })
        .collect::<Vec<_>>();

    bail_if_empty!(data, Ok(StatusCode::NO_CONTENT.into_response()));
    Ok((StatusCode::OK, Json(data)).into_response())
}
