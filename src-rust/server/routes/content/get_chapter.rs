use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::types::BasePageResponse;
use crate::{routes::errors::InternalError, types::id::UserID, utils::app_state::AppState};

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct ChapterResponse {
    id: String,
    number: i32,
    description: Option<String>,

    pages: Option<Vec<BasePageResponse>>,
}

/// Chapter info & pages
#[utoipa::path(get, path = "/api/content/chapter/{chapter_id}", responses(
    (status = 200, description = "Fetch chapter success", body = ChapterResponse),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "No chapter found for the given id"),
    (status = 500, description = "Internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_chapter(
    State(app_state): State<Arc<AppState>>,
    Path(chapter_id): Path<i64>,
    Extension(_user_id): Extension<UserID>,
) -> Result<Response, InternalError> {
    let Some(body) = sqlx::query!(
        r#"SELECT c.id AS id,
            c.number AS number,
            c.description AS description,
            ARRAY_AGG(
                json_build_object(
                    'id',
                    cp.id,
                    'description',
                    cp.description
                )
            ) AS "pages"
        FROM chapters c
            LEFT JOIN chapters_pages cp ON cp.chapter_id = c.id
        WHERE c.id = $1
        GROUP BY c.id"#,
        chapter_id,
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        InternalError::DB(e)
    })?
    .map(|r| ChapterResponse {
        id: r.id.to_string(),
        number: r.number,
        description: r.description,
        pages: r.pages.map(|rs| {
            rs.into_iter()
                .filter_map(|r| {
                    let page = r.as_object()?;

                    Some(BasePageResponse {
                        id: page.get("id")?.as_str()?.to_string(),
                        blurhash: None,
                        width: None,
                        height: None,
                        jxl: false,
                        description: page
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|d| d.to_string()),
                    })
                })
                .collect()
        }),
    }) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if body.pages.as_ref().filter(|p| p.is_empty()).is_none() {
        return Err(InternalError::ChapterNoPage(chapter_id));
    }

    Ok((StatusCode::OK, Json(body)).into_response())
}
