use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
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
    let (title_id, path_in_content_file) =
        match sqlx::query!("SELECT id, path FROM pages WHERE id = $1", page_id)
            .fetch_optional(&app_state.pool)
            .await
            .context("can't find page")?
        {
            Some(page) => (page.id, page.path),
            None => {
                return Ok((StatusCode::NOT_FOUND, "page not found".to_string()).into_response())
            }
        };

    // TODO: join these queries
    let path = match sqlx::query!("SELECT path FROM titles WHERE id = $1", title_id)
        .fetch_optional(&app_state.pool)
        .await
        .context("can't find title")?
    {
        Some(title) => title.path,
        None => return Ok((StatusCode::NOT_FOUND, "title not found".to_string()).into_response()),
    };

    let page_file_buf = ArchiveFile::from(PathBuf::from(&path))
        .context("can't create ArchiveFile from content file")
        .and_then(|mut archive_file| archive_file.read_file(&path_in_content_file))
        .context("can't get page file from content file")?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            match path_in_content_file.split('.').last().unwrap_or_default() {
                "jpg" => "image/jpeg".to_string(),
                v => format!("image/{v}"),
            },
        )],
        page_file_buf,
    )
        .into_response())
}
