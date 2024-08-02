use std::sync::Arc;

use crate::{models::prelude::*, AppError, AppState};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::*;
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
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn get_categories(State(data): State<Arc<AppState>>) -> Result<Response, AppError> {
    let data = Categories::find()
        .all(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find categories: {}", e)))?
        .into_iter()
        .map(|category| CategoryResponse {
            id: category.id.to_string(),
            name: category.name,
            description: category.description,
        })
        .collect();

    Ok((StatusCode::OK, Json(CategoriesResponseBody { data })).into_response())
}
