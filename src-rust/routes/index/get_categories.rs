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
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct CategoriesResponseBody {
    pub data: Vec<CategoryResponse>,
}

/// Get all categories to be displayed on the library page.
#[utoipa::path(get, path = "/api/index/categories", responses(
    (status = 200, description = "Fetch all categories successful", body = CategoriesResponseBody),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn get_categories(State(app_state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let data = sqlx::query!("SELECT id, name, description FROM categories")
        .fetch_all(&app_state.pool)
        .await
        .context("can't find categories")?
        .into_iter()
        .map(|category| CategoryResponse {
            id: category.id,
            name: category.name,
            description: category.description,
        })
        .collect::<Vec<_>>();

    Ok((StatusCode::OK, Json(CategoriesResponseBody { data })).into_response())
}
