use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use crate::{models::prelude::*, AppError, AppState};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use zip::ZipArchive;

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

    let mut buffer = Vec::new();
    ZipArchive::new(
        File::open(title_in_db.path)
            .map_err(|e| AppError::from(anyhow::anyhow!("open zip file error: {}", e)))?,
    )
    .map_err(|e| AppError::from(anyhow::anyhow!("read zip file error: {}", e)))?
    .by_name(&path_in_content_file)
    .map_err(|e| AppError::from(anyhow::anyhow!("get page file error: {}", e)))?
    .read_to_end(&mut buffer)
    .map_err(|e| AppError::from(anyhow::anyhow!("read page file error: {}", e)))?;

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
        buffer,
    )
        .into_response())
}
