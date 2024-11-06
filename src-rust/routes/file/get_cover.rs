use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    body::Body,
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
    let record = match sqlx::query!(
        "SELECT titles.path as title_file_path, cover_path, pages.filesize as cover_size
        FROM titles
            LEFT JOIN pages ON pages.title_id = titles.id
            AND pages.path = cover_path
        WHERE titles.id = $1",
        title_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't find title path")?
    {
        Some(record) => record,
        None => return Ok((StatusCode::NOT_FOUND, "title not found".to_string()).into_response()),
    };

    let cover_path = match record.cover_path {
        Some(path) => path,
        None => {
            return Ok((StatusCode::NOT_FOUND, "title has no cover".to_string()).into_response())
        }
    };

    let headers = [(
        header::CONTENT_TYPE,
        // TODO: works but too janky
        match cover_path.split('.').last().unwrap_or_default() {
            "" => "image".to_string(),
            "jpg" => "image/jpeg".to_string(),
            v => format!("image/{v}"),
        },
    )];

    let content = Body::from_stream(
        ArchiveFile::from(PathBuf::from(record.title_file_path))?
            .stream_file(cover_path, record.cover_size)?,
    );

    Ok((StatusCode::OK, headers, content).into_response())
}
