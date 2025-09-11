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
    utils::{archive_file::ArchiveFile, constants::GET_COVER_FILE_PATH, result_utils::ResultUtils},
};

/// Get cover file for title
#[utoipa::path(
    get,
    path = GET_COVER_FILE_PATH,
    responses(
        (status = 200, description = "Fetch cover successful", body = Vec<u8>),
        (status = 204, description = "Title/chapter not found or has no cover"),
        (status = 500, description = "Internal server error", body = String),
    ),
    params(
        ("title_or_chapter_id" = String, Path, description = "Title or Chapter ID")
    )
)]
pub async fn get_cover_file(
    State(app_state): State<Arc<AppState>>,
    Path(title_or_chapter_id): Path<String>,
) -> Result<Response, InternalError> {
    let Some((parent_path, page_path, page_filesize)) = sqlx::query!(
        "SELECT
            p.path AS page_path,
            t.path AS title_path,
            c.path AS chapter_path,
            p.size AS page_size
        FROM
            pages p
            JOIN title_covers tc ON p.id = tc.page_id
            JOIN titles t ON tc.title_id = t.id
            JOIN chapters c ON t.id = c.title_id
        WHERE
            tc.title_id = ?
        UNION
        SELECT
            p.path AS page_path,
            t.path AS title_path,
            c.path AS chapter_path,
            p.size AS page_size
        FROM
            pages p
            JOIN chapter_covers cc ON p.id = cc.page_id
            JOIN chapters c ON cc.chapter_id = c.id
            JOIN titles t ON c.title_id = t.id
        WHERE
            cc.chapter_id = ?",
        title_or_chapter_id,
        title_or_chapter_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't query title/chapter cover: {e}"))?
    .map(|r| {
        let parent_path = app_state
            .config
            .library_path
            .as_ref()
            .join(r.title_path)
            .join(r.chapter_path);

        (parent_path, r.page_path, r.page_size)
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

    if let Some(size) = page_filesize
        .and_then(|s| {
            s.parse::<u64>().okay(|e| {
                warn!(
                    "can't parse page size for {page_path} in {}: {e:?}",
                    parent_path.display()
                );
            })
        })
        .map(HeaderValue::from)
    {
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
