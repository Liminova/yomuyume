use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{
    archive_file::ArchiveFile,
    utils::{app_error::AppError, app_state::AppState},
};

/// get cover
///
/// get the content of a cover file for a title
#[utoipa::path(get, path = "/api/file/cover/{title_id}", responses(
    (status = 200, description = "fetch cover successful", body = Vec<u8>),
    (status = 204, description = "title has no cover"),
    (status = 401, description = "unauthorized", body = String),
    (status = 404, description = "title not found"),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_cover(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<i64>,
) -> Result<Response, AppError> {
    let record = match sqlx::query!(
        "SELECT
            titles.path AS title_path,
            cover_path,
            oneshots_pages.filesize AS cover_filesize,
            titles.is_dir AS title_is_dir
        FROM titles
            LEFT JOIN oneshots_pages ON oneshots_pages.title_id = titles.id
            AND oneshots_pages.path = cover_path
        WHERE titles.id = $1",
        title_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query title path")?
    {
        Some(record) => record,
        None => return Ok((StatusCode::NOT_FOUND).into_response()),
    };

    let cover_path = match record.cover_path {
        Some(path) => path,
        None => return Ok((StatusCode::NO_CONTENT).into_response()),
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

    let content = match record.title_is_dir {
        true => {
            let file_path = PathBuf::from(record.title_path).join(cover_path);
            let file = File::open(file_path)
                .await
                .context("can't open page file")?;
            let stream = ReaderStream::new(file);

            Body::from_stream(stream)
        }
        false => {
            let archive_file = PathBuf::from(record.title_path);
            let stream =
                archive_file.stream_file_from_archive(cover_path, record.cover_filesize)?;

            Body::from_stream(stream)
        }
    };

    Ok((StatusCode::OK, headers, content).into_response())
}
