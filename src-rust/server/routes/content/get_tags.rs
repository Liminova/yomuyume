use std::sync::Arc;

use crate::{
    routes::errors::InternalErr,
    utils::{app_state::AppState, constants::GET_TAGS_PATH},
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
#[utoipa::path(get, path = GET_TAGS_PATH, responses(
    (status = 200, description = "Get tags success", body = TagsResponse),
    (status = 204, description = "No tag found"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_tags(State(app_state): State<Arc<AppState>>) -> Result<Response, InternalErr> {
    let data = sqlx::query!("SELECT id, name FROM tags")
        .fetch_all(&app_state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::DB(e)
        })?
        .into_iter()
        .map(|record| InnerTagResponse {
            id: record.id.to_string(),
            name: record.name,
        })
        .collect::<Vec<_>>();

    if data.is_empty() {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }
    Ok((StatusCode::OK, Json(data)).into_response())
}
