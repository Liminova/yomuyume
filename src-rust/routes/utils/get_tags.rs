use std::sync::Arc;

use crate::{AppError, AppState};

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct TagResponseBody {
    pub id: String,
    pub name: String,
}

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize)]
pub struct TagsMapResponseBody {
    pub data: Vec<TagResponseBody>,
}

#[utoipa::path(get, path = "/api/utils/tags", responses(
    (status = 200, description = "tags map", body = Vec<TagResponseBody>),
    (status = 204, description = "no tags found"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_tags(State(app_state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let data = sqlx::query!("SELECT id, name FROM tags")
        .fetch_all(&app_state.pool)
        .await
        .context("can't fetch tags")?
        .into_iter()
        .map(|record| TagResponseBody {
            id: record.id.to_string(),
            name: record.name,
        })
        .collect::<Vec<_>>();

    Ok((StatusCode::OK, Json(TagsMapResponseBody { data })).into_response())
}
