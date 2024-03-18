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

#[utoipa::path(get, path = "/api/file/cover/{id}", responses(
    (status = 200, description = "Fetch cover successful", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 404, description = "Cover not found", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
))]
pub async fn get_cover(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<String>,
    header: HeaderMap,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let cover_model = Covers::find()
        .filter(covers::Column::Id.eq(title_id))
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.not_found("Cover not found."))?;

    let title_model = Titles::find()
        .filter(titles::Column::Id.contains(&cover_model.id))
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
        .by_name(cover_model.path.as_ref())
        .map_err(|e| builder.internal(format!("Read cover from zip error: {}", e)))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| builder.internal(format!("Read cover error: {}", e)))?;

    let mime_type = format!(
        "image/{}",
        PathBuf::from(cover_model.path)
            .extension()
            .map(|s| s.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_ascii_lowercase()
    );

    Ok((StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], buffer))
}
