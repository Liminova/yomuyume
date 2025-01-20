use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::{app_error::AppError, app_state::AppState, macros::bail_if_empty};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct CategoryResponseBody {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// get categories
///
/// with their information
#[utoipa::path(get, path = "/api/content/categories", responses(
    (status = 200, description = "fetch all categories successful", body = Vec<CategoryResponseBody>),
    (status = 204, description = "no category found"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_categories(State(app_state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let data = sqlx::query!("SELECT id, name, description FROM categories")
        .fetch_all(&app_state.pool)
        .await
        .context("can't query categories")?
        .into_iter()
        .map(|category| CategoryResponseBody {
            id: category.id.to_string(),
            name: category.name,
            description: category.description,
        })
        .collect::<Vec<_>>();

    bail_if_empty!(data, Ok(StatusCode::NO_CONTENT.into_response()));
    Ok((StatusCode::OK, Json(data)).into_response())
}
