use std::sync::Arc;

use crate::{models::prelude::*, AppError, AppState};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagResponseBody {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagsMapResponseBody {
    pub data: Vec<TagResponseBody>,
}

#[utoipa::path(get, path = "/api/utils/tags", responses(
    (status = 200, description = "Tags map.", body = TagsMapResponseBody),
    (status = 500, description = "Internal server error.", body = String),
))]
pub async fn get_tags(State(app_state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let tags = Tags::find().all(&app_state.db).await.map_err(AppError::from)?;

    let data = tags
        .into_iter()
        .map(|tag| TagResponseBody {
            id: tag.id,
            name: tag.name,
        })
        .collect::<Vec<_>>();

    Ok((StatusCode::OK, Json(TagsMapResponseBody { data })).into_response())
}
