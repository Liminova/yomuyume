use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::{error, warn};

use crate::{
    AppState,
    routes::errors::InternalError,
    utils::{archive_file::ArchiveFile, constants::GET_PAGE_FILE_PATH, result_utils::ResultUtils},
};

/// Get page file
#[utoipa::path(
    get,
    path = GET_PAGE_FILE_PATH,
    responses(
        (status = 200, description = "Fetch page successful", body = Vec<u8>),
        (status = 404, description = "Page not found"),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("title_id" = String, Path, description = "Title ID"),
        ("chapter_id" = String, Path, description = "Chapter ID"),
        ("page_number" = u32, Path, description = "Page number"),
    )
)]
pub async fn get_page_file(
    State(app_state): State<Arc<AppState>>,
    Path((title_id, chapter_id, page_number)): Path<(String, String, u32)>,
) -> Result<Response, InternalError> {
    let Some((parent_path, page_path, page_filesize)) = sqlx::query!(
        "SELECT p.path AS page_path,
            c.path AS chapter_path,
            t.path AS title_path,
            p.size AS size
        FROM
            pages p
            JOIN chapters c ON p.chapter_id = c.id
            JOIN titles t ON c.title_id = t.id
        WHERE
            t.id = ?
            AND c.id = ?
            AND p.page_number = ?",
        title_id,
        chapter_id,
        page_number
    )
    .fetch_optional(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't query page file: {e}"))?
    .map(|r| {
        let parent_path = app_state
            .config
            .library_path
            .as_ref()
            .join(r.title_path)
            .join(r.chapter_path);

        (parent_path, r.page_path, r.size)
    }) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let mut headers = header::HeaderMap::new();
    if let Some(mime) = page_path
        .split('.')
        .next_back()
        .and_then(mimatcher::mimatcher)
        .map(HeaderValue::from_static)
    {
        headers.insert(header::CONTENT_TYPE, mime);
    }

    if let Some(size) = page_filesize.and_then(|s| {
        s.parse::<u64>()
            .okay(|e| {
                warn!(
                    "can't parse page size for {}::{}: {e}",
                    parent_path.display(),
                    page_path
                );
            })
            .map(HeaderValue::from)
    }) {
        headers.insert(header::CONTENT_LENGTH, size);
    }

    let body = if parent_path.is_dir() {
        let file_path = parent_path.join(page_path);

        let stream = File::open(&file_path)
            .await
            .inspect_err(|e| error!("can't open page file {file_path:?}: {e}"))
            .map(ReaderStream::new)?;

        Body::from_stream(stream)
    } else {
        let stream = parent_path
            .stream_file_from_archive(&page_path)
            .await
            .inspect_err(|e| error!("can't open page file {parent_path:?}::{page_path}: {e}"))
            .map(ReaderStream::new)?;

        Body::from_stream(stream)
    };

    Ok((StatusCode::OK, headers, body).into_response())
}
