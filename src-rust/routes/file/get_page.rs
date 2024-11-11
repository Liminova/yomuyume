use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{AppError, AppState, ArchiveFile};

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
    let record = match sqlx::query!(
        "SELECT
            pages.path AS page_path,
            titles.path AS title_path,
            pages.filesize AS page_filesize
        FROM pages JOIN titles
            ON pages.title_id = titles.id
        WHERE pages.id = $1",
        page_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query page")?
    {
        Some(record) => record,
        None => return Ok((StatusCode::NOT_FOUND).into_response()),
    };

    let headers = [(
        header::CONTENT_TYPE,
        match record.page_path.split('.').last().unwrap_or_default() {
            "" => "image".to_string(),
            "jpg" => "image/jpeg".to_string(),
            v => format!("image/{v}"),
        },
    )];

    let content = Body::from_stream(
        PathBuf::from(record.title_path).stream_file(record.page_path, record.page_filesize)?,
    );

    Ok((StatusCode::OK, headers, content).into_response())
}
