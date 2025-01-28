use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::utils::{app_error::AppError, app_state::AppState, archive_file::ArchiveFile};

/// get cover
///
/// get the content of a cover file for a title
#[utoipa::path(get, path = "/api/file/cover/{title_id}", responses(
    (status = 200, description = "fetch cover successful", body = Vec<u8>),
    (status = 204, description = "title has no cover"),
    (status = 401, description = "unauthorized", body = String),
    (status = 404, description = "title not found"),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_cover(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<i64>,
) -> Result<Response, AppError> {
    let Some(title) = sqlx::query!(
        "SELECT path,
            cover_path,
            is_dir,
            is_series
        FROM titles
        WHERE id = $1",
        title_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        AppError::DB(e)
    })?
    else {
        return Ok((StatusCode::NOT_FOUND).into_response());
    };

    let Some(cover_path) = title.cover_path else {
        return Ok((StatusCode::NO_CONTENT).into_response());
    };

    let headers = [(
        header::CONTENT_TYPE,
        match cover_path
            .split('.')
            .last()
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "" => "image".to_string(),
            "jpg" => "image/jpeg".to_string(),
            v => format!("image/{v}"),
        },
    )];

    let content = match (title.is_series, title.is_dir) {
        (false, true) => File::open(
            app_state
                .config
                .library_path
                .as_ref()
                .join(title.path)
                .join(cover_path),
        )
        .await
        .map_err(|e| {
            tracing::error!("1SHOT DIR {title_id}: {e:?}");
            AppError::IO(e)
        })
        .map(ReaderStream::new)
        .map(Body::from_stream)?,

        (false, false) => app_state
            .config
            .library_path
            .as_ref()
            .join(title.path)
            .stream_file_from_archive(cover_path, None)
            .map_err(|e| {
                tracing::error!("1SHOT ARCHIVE {title_id}: {e:?}");
                AppError::Archive(e)
            })
            .map(Body::from_stream)?,

        (true, true) => match cover_path
            .split_once('/')
            .map(|(a, b)| (a.to_string(), b.to_string()))
        {
            Some((chapter_part, page_part)) => {
                let chapter_path = app_state
                    .config
                    .library_path
                    .as_ref()
                    .join(title.path)
                    .join(chapter_part);

                if chapter_path.is_dir() {
                    File::open(chapter_path.join(page_part))
                        .await
                        .map_err(|e| {
                            tracing::error!("SERIES CHAPTER ARCHIVE {title_id}: {e:?}");
                            AppError::IO(e)
                        })
                        .map(ReaderStream::new)
                        .map(Body::from_stream)?
                } else {
                    chapter_path
                        .stream_file_from_archive(page_part, None)
                        .map_err(|e| {
                            tracing::error!("SERIES CHAPTER ARCHIVE {title_id}: {e:?}");
                            AppError::Archive(e)
                        })
                        .map(Body::from_stream)?
                }
            }
            None => File::open(
                app_state
                    .config
                    .library_path
                    .as_ref()
                    .join(title.path)
                    .join(&cover_path),
            )
            .await
            .map_err(|e| {
                tracing::error!("SERIES ROOT {title_id}: {e:?}");
                AppError::IO(e)
            })
            .map(ReaderStream::new)
            .map(Body::from_stream)?,
        },

        (true, false) => {
            unreachable!()
        }
    };

    Ok((StatusCode::OK, headers, content).into_response())
}
