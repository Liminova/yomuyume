use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{parse_pages, structs::BasePageResponse};
use crate::{
    routes::errors::InternalErr,
    structs::id::UserID,
    utils::{app_state::AppState, constants::GET_CHAPTER_PATH},
};

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct ChapterResponse {
    id: String,
    number: i32,
    description: Option<String>,

    pages: Option<Vec<BasePageResponse>>,
}

/// Chapter info & pages
#[utoipa::path(get, path = GET_CHAPTER_PATH, responses(
    (status = 200, description = "Fetch chapter success", body = ChapterResponse),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "No chapter found for the given id"),
    (status = 500, description = "Internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_chapter(
    State(app_state): State<Arc<AppState>>,
    Path(chapter_id): Path<i64>,
    Extension(_user_id): Extension<UserID>,
) -> Result<Response, InternalErr> {
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
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?
    .map(|r| ChapterResponse {
        id: r.id.to_string(),
        number: r.number,
        description: r.description,
        pages: r.pages.map(parse_pages),
    }) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if body.pages.as_ref().filter(|p| p.is_empty()).is_none() {
        return Err(InternalErr::ChapterNoPage(chapter_id));
    }

    Ok((StatusCode::OK, Json(body)).into_response())
}
