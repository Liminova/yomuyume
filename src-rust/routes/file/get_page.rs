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

/// get page
///
/// get the content of a page file for a title
#[utoipa::path(get, path = "/api/file/page/{page_id}", responses(
    (status = 200, description = "fetch page successful", body = Vec<u8>),
    (status = 401, description = "unauthorized", body = String),
    (status = 404, description = "page not found", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_page(
    State(app_state): State<Arc<AppState>>,
    Path(page_id): Path<i64>,
) -> Result<Response, AppError> {
    let Some(record) = sqlx::query!(
        r#"SELECT
            p.path AS page_path,
            t.path AS title_path,
            p.filesize AS page_filesize,
            t.is_dir AS "title_is_dir!"
        FROM oneshots_pages p JOIN titles t
            ON p.title_id = t.id
        WHERE p.id = $1"#,
        page_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query page")?
    else {
        return Ok((StatusCode::NOT_FOUND).into_response());
    };

    let headers = [(
        header::CONTENT_TYPE,
        match record.page_path.split('.').last().unwrap_or_default() {
            "" => "image".to_string(),
            "jpg" => "image/jpeg".to_string(),
            v => format!("image/{v}"),
        },
    )];

    let content = if record.title_is_dir {
        let file_path = PathBuf::from(record.title_path).join(record.page_path);
        let file = File::open(file_path)
            .await
            .context("can't open page file")?;
        let stream = ReaderStream::new(file);

        Body::from_stream(stream)
    } else {
        let archive_file = PathBuf::from(record.title_path);
        let stream =
            archive_file.stream_file_from_archive(record.page_path, record.page_filesize)?;

        Body::from_stream(stream)
    };

    Ok((StatusCode::OK, headers, content).into_response())
}
