use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use crate::{
    routes::errors::InternalErr,
    utils::{app_state::AppState, constants::GET_CATEGORIES_PATH, macros::bail_if_empty},
};

#[skip_serializing_none]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct InnerCategoriesResponse {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,

    pub cover_blurhash: Option<String>,
    pub cover_width: Option<i32>,
    pub cover_height: Option<i32>,
    pub cover_jxl: Option<bool>,
}

type CategoriesResponse = Vec<InnerCategoriesResponse>;

/// Categories & infos
#[utoipa::path(get, path = GET_CATEGORIES_PATH, responses(
    (status = 200, description = "Fetch all categories success", body = CategoriesResponse),
    (status = 204, description = "No category found"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_categories(
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, InternalErr> {
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
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?
    .into_iter()
    .map(|record| InnerCategoriesResponse {
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
