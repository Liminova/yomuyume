use crate::{models::prelude::*, routes::ErrRsp, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::EntityTrait;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Debug, ToSchema)]
pub struct TagsMapResponseBody {
    pub data: Vec<(u32, String)>,
}

#[utoipa::path(get, path = "/api/utils/tags", responses(
    (status = 200, description = "Tags map.", body = TagsMapResponseBody),
    (status = 500, description = "Internal server error.", body = ErrorResponseBody),
))]
pub async fn get_tags(State(data): State<Arc<AppState>>) -> Result<impl IntoResponse, ErrRsp> {
    let tags = Tags::find()
        .all(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't get tags: {}", e)))?;

    let data = tags
        .into_iter()
        .map(|tag| (tag.id, tag.name))
        .collect::<Vec<(u32, String)>>();

    Ok((StatusCode::OK, Json(TagsMapResponseBody { data })))
}
