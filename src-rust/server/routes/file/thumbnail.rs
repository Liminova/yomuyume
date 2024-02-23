use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use zip::ZipArchive;

use crate::{models::prelude::*, routes::ErrRsp, AppState};

#[utoipa::path(get, path = "/api/file/thumbnail/{thumbnail_id}", responses(
    (status = 200, description = "Fetch thumbnail successful", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 404, description = "Thumbnail not found", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody),
))]
pub async fn get_thumbnail(
    State(data): State<Arc<AppState>>,
    Path(thumbnail_id): Path<String>,
) -> Result<impl IntoResponse, ErrRsp> {
    let thumbnail_model = Thumbnails::find()
        .filter(thumbnails::Column::Id.eq(thumbnail_id))
        .one(&data.db)
        .await
        .map_err(ErrRsp::db)?
        .ok_or_else(|| ErrRsp::not_found("Page not found."))?;

    let title_model = Titles::find()
        .filter(titles::Column::Id.contains(&thumbnail_model.id))
        .one(&data.db)
        .await
        .map_err(ErrRsp::db)?
        .ok_or_else(|| ErrRsp::not_found("Title not found."))?;
    let mut zip = ZipArchive::new(
        File::open(title_model.path).map_err(|e| ErrRsp::internal(format!("File error: {}", e)))?,
    )
    .map_err(|e| ErrRsp::internal(format!("Zip error: {}", e)))?;

    let mut file = zip
        .by_name(thumbnail_model.path.as_ref())
        .map_err(|e| ErrRsp::internal(format!("Zip error: {}", e)))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| ErrRsp::internal(format!("File read error: {}", e)))?;

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
