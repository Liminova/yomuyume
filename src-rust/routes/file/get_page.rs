use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

use crate::{models::prelude::*, AppError, AppState, ArchiveFile};

#[utoipa::path(get, path = "/api/file/page/{page_id}", responses(
    (status = 200, description = "Fetch page successful.", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "Page not found", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_page(
    State(app_state): State<Arc<AppState>>,
    Path(page_id): Path<String>,
) -> Result<Response, AppError> {
    let (title_id, path_in_content_file) = match Pages::find()
        .select_only()
        .columns(vec![pages::Column::TitleId, pages::Column::Path])
        .filter(pages::Column::Id.contains(page_id))
        .into_tuple::<(String, String)>()
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find page: {}", e)))?
    {
        Some(page) => page,
        None => return Ok((StatusCode::NOT_FOUND, "page not found".to_string()).into_response()),
    };

    let title_in_db = match Titles::find()
        .filter(titles::Column::Id.contains(&title_id))
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find title: {}", e)))?
    {
        Some(title) => title,
        None => return Ok((StatusCode::NOT_FOUND, "title not found".to_string()).into_response()),
    };

    let page_file_buf = ArchiveFile::from(PathBuf::from(title_in_db.path))
        .context("can't create ArchiveFile from content file")
        .and_then(|mut archive_file| archive_file.get_file(&path_in_content_file))
        .context("can't get page file from content file")?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            format!(
                "image/{}",
                PathBuf::from(path_in_content_file)
                    .extension()
                    .map(|s| s.to_str().unwrap_or(""))
                    .unwrap_or("")
                    .to_ascii_lowercase()
            ),
        )],
        page_file_buf,
    )
        .into_response())
}
