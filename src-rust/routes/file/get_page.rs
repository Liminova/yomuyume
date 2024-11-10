use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{AppError, AppState, ArchiveFile};

#[utoipa::path(get, path = "/api/file/page/{page_id}", responses(
    (status = 200, description = "Fetch page successful.", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "Page not found", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_page(
    State(app_state): State<Arc<AppState>>,
    Path(page_id): Path<i64>,
) -> Result<Response, AppError> {
    let result = sqlx::query!(
        r#"
        SELECT
            pages.path AS page_path,
            titles.path AS title_path,
            pages.filesize AS page_filesize
        FROM pages JOIN titles
            ON pages.title_id = titles.id
        WHERE pages.id = $1
        "#,
        page_id
    )
    .fetch_one(&app_state.pool)
    .await
    .context("title not found from page id")?;

    let headers = [(
        header::CONTENT_TYPE,
        match result.page_path.split('.').last().unwrap_or_default() {
            "" => "image".to_string(),
            "jpg" => "image/jpeg".to_string(),
            v => format!("image/{v}"),
        },
    )];

    let content = Body::from_stream(
        PathBuf::from(result.title_path).stream_file(result.page_path, result.page_filesize)?,
    );

    Ok((StatusCode::OK, headers, content).into_response())
}
