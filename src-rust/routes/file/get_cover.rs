use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{AppError, AppState, ArchiveFile};

#[utoipa::path(get, path = "/api/file/cover/{id}", responses(
    (status = 200, description = "Fetch cover successful", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "Cover not found", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_cover(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<i64>,
) -> Result<Response, AppError> {
    let (content_file_path, cover_path) = match sqlx::query!(
        "SELECT path, cover_path FROM titles WHERE id = $1",
        title_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't find title path")?
    {
        Some(result) => (result.path, result.cover_path),
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
        .and_then(|mut archive_file| archive_file.read_file(&cover_path))
        .context("can't get cover file from content file")?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            match cover_path.split('.').last().unwrap_or_default() {
                "jpg" => "image/jpeg".to_string(),
                v => format!("image/{v}"),
            },
        )],
        cover_file_buf,
    )
        .into_response())
}
