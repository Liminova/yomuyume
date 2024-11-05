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
    (status = 200, description = "Tags map.", body = TagsMapResponseBody),
    (status = 500, description = "Internal server error.", body = String),
))]
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
