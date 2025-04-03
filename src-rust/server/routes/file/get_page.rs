use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{
    routes::errors::InternalErr,
    utils::{app_state::AppState, archive_file::ArchiveFile, constants::GET_PAGE_PATH},
};

/// Get page file
#[utoipa::path(
    get,
    path = GET_PAGE_PATH,
    responses(
        (status = 200, description = "Fetch page successful", body = Vec<u8>),
        (status = 401, description = "Unauthorized", body = String),
        (status = 404, description = "Page not found", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-id" = [], "session-secret" = [])))
]
pub async fn get_page(
    State(app_state): State<Arc<AppState>>,
    Path((is_series, page_id)): Path<(bool, i64)>,
) -> Result<Response, InternalErr> {
    let Some((parent_path, page_path, page_filesize, title_is_dir)) = (if is_series {
        sqlx::query!(
            r#"SELECT p.path AS page_path,
                c.path AS chapter_path,
                t.path AS title_path,
                p.filesize AS page_filesize,
                t.is_dir AS "title_is_dir!"
            FROM chapters_pages p
                JOIN chapters c ON p.chapter_id = c.id
                JOIN titles t ON c.title_id = t.id
            WHERE p.id = $1"#,
            page_id
        )
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::DB(e)
        })?
        .map(|r| {
            (
                app_state
                    .config
                    .library_path
                    .as_ref()
                    .join(r.title_path)
                    .join(r.chapter_path),
                r.page_path,
                r.page_filesize,
                r.title_is_dir,
            )
        })
    } else {
        sqlx::query!(
            r#"SELECT p.path AS page_path,
                t.path AS title_path,
                p.filesize AS page_filesize,
                t.is_dir AS "title_is_dir!"
            FROM oneshots_pages p
                JOIN titles t ON p.title_id = t.id
            WHERE p.id = $1"#,
            page_id
        )
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::DB(e)
        })?
        .map(|r| {
            (
                app_state.config.library_path.as_ref().join(r.title_path),
                r.page_path,
                r.page_filesize,
                r.title_is_dir,
            )
        })
    }) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let header = [(
        header::CONTENT_TYPE,
        match page_path.split('.').next_back().unwrap_or_default() {
            "" => "image".to_string(),
            "jpg" => "image/jpeg".to_string(),
            v => format!("image/{v}"),
        },
    )];

    let body = if title_is_dir {
        let file_path = parent_path.join(page_path);
        let file = File::open(file_path).await.map_err(|e| {
            tracing::error!("{e}");
            InternalErr::IO(e)
        })?;
        let stream = ReaderStream::new(file);

        Body::from_stream(stream)
    } else {
        let stream = parent_path
            .stream_file_from_archive(page_path, page_filesize)
            .map_err(|e| {
                tracing::error!("{e}");
                InternalErr::Archive(e)
            })?;

        Body::from_stream(stream)
    };

    Ok((StatusCode::OK, header, body).into_response())
}
