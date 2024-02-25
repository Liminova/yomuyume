use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use crate::{
    models::prelude::*,
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use zip::ZipArchive;

#[utoipa::path(get, path = "/api/file/thumbnail/{id}", responses(
    (status = 200, description = "Fetch thumbnail successful", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 404, description = "Thumbnail not found", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
))]
pub async fn get_thumbnail(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<String>,
    header: HeaderMap,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let thumbnail_model = Thumbnails::find()
        .filter(thumbnails::Column::Id.eq(title_id))
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.not_found("Thumbnail not found."))?;

    let title_model = Titles::find()
        .filter(titles::Column::Id.contains(&thumbnail_model.id))
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.not_found("Title not found."))?;

    let mut zip = ZipArchive::new(
        File::open(title_model.path)
            .map_err(|e| builder.internal(format!("Read title error: {}", e)))?,
    )
    .map_err(|e| builder.internal(format!("Zip error: {}", e)))?;

    let mut file = zip
        .by_name(thumbnail_model.path.as_ref())
        .map_err(|e| builder.internal(format!("Read thumbnail from zip error: {}", e)))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| builder.internal(format!("Read thumbnail error: {}", e)))?;

    let mime_type = format!(
        "image/{}",
        PathBuf::from(thumbnail_model.path)
            .extension()
            .map(|s| s.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_ascii_lowercase()
    );

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], buffer))
}
