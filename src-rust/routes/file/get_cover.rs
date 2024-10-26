use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

use crate::{models::prelude::*, AppError, AppState, ArchiveFile};

#[utoipa::path(get, path = "/api/file/cover/{id}", responses(
    (status = 200, description = "Fetch cover successful", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "Cover not found", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_cover(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<String>,
) -> Result<Response, AppError> {
    let (content_file_path, cover_path) = match Titles::find()
        .select_only()
        .columns(vec![titles::Column::Path, titles::Column::CoverPath])
        .filter(titles::Column::Id.eq(title_id))
        .into_tuple::<(String, Option<String>)>()
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find title path: {}", e)))?
    {
        Some(result) => result,
        None => return Ok((StatusCode::NOT_FOUND, "title not found".to_string()).into_response()),
    };

    let cover_path = match cover_path {
        Some(path) => path,
        None => {
            return Ok((StatusCode::NOT_FOUND, "title has no cover".to_string()).into_response())
        }
    };

    let cover_file_buf = ArchiveFile::from(PathBuf::from(content_file_path))
        .context("can't create ArchiveFile from content file")
        .and_then(|mut archive_file| archive_file.get_file(&cover_path))
        .context("can't get cover file from content file")?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            format!(
                "image/{}",
                PathBuf::from(cover_path)
                    .extension()
                    .map(|s| s.to_str().unwrap_or(""))
                    .unwrap_or("")
                    .to_ascii_lowercase()
            ),
        )],
        cover_file_buf,
    )
        .into_response())
}
