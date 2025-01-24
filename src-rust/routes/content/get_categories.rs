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
    pub name: Option<String>,
    pub description: Option<String>,

    pub cover_blurhash: Option<String>,
    pub cover_width: Option<i32>,
    pub cover_height: Option<i32>,
    pub cover_jxl: Option<bool>,
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
    let data = sqlx::query!(
        "SELECT id,
            name,
            description,
            cover_path,
            cover_blurhash,
            cover_width,
            cover_height
        FROM categories"
    )
    .fetch_all(&app_state.pool)
    .await
    .context("can't query categories")?
    .into_iter()
    .map(|record| CategoryResponseBody {
        id: record.id.to_string(),
        name: record.name,
        description: record.description,

        cover_blurhash: record.cover_blurhash,
        cover_width: record.cover_width,
        cover_height: record.cover_height,
        cover_jxl: record.cover_path.map(|path| {
            std::path::Path::new(&path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
        }),
    })
    .collect::<Vec<_>>();

    bail_if_empty!(data, Ok(StatusCode::NO_CONTENT.into_response()));
    Ok((StatusCode::OK, Json(data)).into_response())
}
