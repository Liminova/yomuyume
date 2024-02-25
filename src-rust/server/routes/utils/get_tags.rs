use std::sync::Arc;

pub use bridge::routes::utils::{TagResponseBody, TagsMapResponseBody};

use crate::{
    models::prelude::*,
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use sea_orm::EntityTrait;

#[utoipa::path(get, path = "/api/utils/tags", responses(
    (status = 200, description = "Tags map.", body = TagsMapResponseBody),
    (status = 500, description = "Internal server error.", body = GenericResponseBody),
))]
pub async fn get_tags(
    State(data): State<Arc<AppState>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let tags = Tags::find()
        .all(&data.db)
        .await
        // .map_err(|e| GenericResponse::internal(format!("Can't get tags: {}", e)))?;
        .map_err(|e| builder.db_error(e))?;

    let data = tags
        .into_iter()
        .map(|tag| TagResponseBody {
            id: tag.id,
            name: tag.name,
        })
        .collect::<Vec<_>>();

    Ok((StatusCode::OK, Json(TagsMapResponseBody { data })))
}
