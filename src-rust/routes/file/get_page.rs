use std::{fs::File, io::Read, path::PathBuf, sync::Arc};

use crate::{models::prelude::*, AppError, AppState};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use zip::ZipArchive;

#[utoipa::path(get, path = "/api/file/page/{page_id}", responses(
    (status = 200, description = "Fetch page successful.", body = Vec<u8>),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "Page not found", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_page(
    State(data): State<Arc<AppState>>,
    Path(page_id): Path<String>,
) -> Result<Response, AppError> {
    let page_in_db = match Pages::find()
        .filter(pages::Column::Id.contains(page_id))
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find page: {}", e)))?
    {
        Some(page) => page,
        None => return Ok((StatusCode::NOT_FOUND, "Page not found.".to_string()).into_response()),
    };

    let title_in_db = match Titles::find()
        .filter(titles::Column::Id.contains(&page_in_db.title_id))
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find title: {}", e)))?
    {
        Some(title) => title,
        None => return Ok((StatusCode::NOT_FOUND, "Title not found.".to_string()).into_response()),
    };

    // zip file -> page file -> buffer
    let mut zip = ZipArchive::new(
        File::open(title_in_db.path)
            .map_err(|e| AppError::from(anyhow::anyhow!("Open zip file error: {}", e)))?,
    )
    .map_err(|e| AppError::from(anyhow::anyhow!("Read zip file error: {}", e)))?;
    let mut file = zip
        .by_name(page_in_db.path.as_ref())
        .map_err(|e| AppError::from(anyhow::anyhow!("Get page file error: {}", e)))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| AppError::from(anyhow::anyhow!("Read page file error: {}", e)))?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            format!(
                "image/{}",
                PathBuf::from(page_in_db.path)
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
