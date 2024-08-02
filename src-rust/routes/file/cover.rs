use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use crate::{models::prelude::*, AppError, AppState};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
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
) -> Result<Response, AppError> {
    let cover_model = match Covers::find()
        .filter(covers::Column::Id.eq(title_id))
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find cover: {}", e)))?
    {
        Some(cover) => cover,
        None => return Ok((StatusCode::NOT_FOUND, "Cover not found.".to_string()).into_response()),
    };

    let title_model_path = match titles::Entity::find()
        .select_only()
        .column(titles::Column::Path)
        .filter(titles::Column::Id.eq(cover_model.id))
        .into_tuple::<String>()
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find title path: {}", e)))?
    {
        Some(path) => path,
        None => return Ok((StatusCode::NOT_FOUND, "Title not found.".to_string()).into_response()),
    };

    // zip file -> cover file -> buffer
    let mut zip = ZipArchive::new(
        File::open(title_model_path)
            .map_err(|e| AppError::from(anyhow::anyhow!("Open zip file error: {}", e)))?,
    )
    .map_err(|e| AppError::from(anyhow::anyhow!("Read zip file error: {}", e)))?;
    let mut file = zip
        .by_name(cover_model.path.as_ref())
        .map_err(|e| AppError::from(anyhow::anyhow!("Get cover file error: {}", e)))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| AppError::from(anyhow::anyhow!("Read cover file error: {}", e)))?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            format!(
                "image/{}",
                PathBuf::from(cover_model.path)
                    .extension()
                    .map(|s| s.to_str().unwrap_or(""))
                    .unwrap_or("")
                    .to_ascii_lowercase()
            ),
        )],
        buffer,
    )
        .into_response())
}
