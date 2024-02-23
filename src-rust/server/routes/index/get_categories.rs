use crate::{models::prelude::*, routes::ErrRsp, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::*;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CategoriesResponseBody {
    /// A list of all categories fetched.
    pub data: Vec<categories::Model>,
}

/// Get all categories to be displayed on the library page.
#[utoipa::path(get, path = "/api/index/categories", responses(
    (status = 200, description = "Fetch all categories successful", body = CategoriesResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn get_categories(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ErrRsp> {
    let data = Categories::find().all(&data.db).await.map_err(ErrRsp::db)?;

    Ok((StatusCode::OK, Json(CategoriesResponseBody { data })))
}
