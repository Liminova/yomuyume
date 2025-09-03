use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use super::is_jxl;
use crate::{
    routes::errors::InternalErr,
    structs::ids::UserID,
    utils::{app_state::AppState, constants::GET_PAGES_PATH},
};

#[skip_serializing_none]
#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct BasePageResponse {
    pub id: String,
    pub blurhash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub jxl: bool,
    pub description: Option<String>,
}

/// Get chapter's pages
#[utoipa::path(
    get,
    path = GET_PAGES_PATH,
    responses(
        (status = 200, description = "Fetch chapter success", body = Vec<BasePageResponse>),
        (status = 401, description = "Unauthorized", body = String),
        (status = 404, description = "No chapter found for the given id"),
        (status = 500, description = "Internal server error", body = String)
    ),
    params(
        ("chapter_id" = i64, Path, description = "Chapter ID")
    ),
    security(("user-id" = [], "session-secret" = [])))
]
pub async fn get_pages(
    State(app_state): State<Arc<AppState>>,
    Path(chapter_id): Path<i64>,
    Extension(_user_id): Extension<UserID>,
) -> Result<Response, InternalErr> {
    let Some(pages) = sqlx::query!(
        r#"SELECT c.id AS id,
            c.number AS number,
            c.description AS description,
            ARRAY_AGG(
                json_build_object(
                    'id',
                    p.id,
                    'description',
                    p.description,
                    'path',
                    p.path
                )
            ) AS "pages"
        FROM chapters c
            LEFT JOIN pages p ON p.chapter_id = c.id
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
    .map(|r| {
        r.pages.map(|vals| {
            let mut pages = vals
                .into_iter()
                .filter_map(|r| {
                    let page = r.as_object()?;
                    let path = page.get("path")?.as_str()?;

                    Some((
                        path.to_string(),
                        BasePageResponse {
                            id: page.get("id")?.as_i64()?.to_string(),
                            blurhash: None,
                            width: None,
                            height: None,
                            jxl: is_jxl(path),
                            description: page
                                .get("description")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string()),
                        },
                    ))
                })
                .collect::<Vec<_>>();

            // sort & dedup by path
            pages.sort_by(|a, b| a.0.cmp(&b.0));
            pages.dedup_by(|a, b| a.0 == b.0);

            pages.into_iter().map(|(_, page)| page).collect::<Vec<_>>()
        })
    }) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if pages.as_ref().filter(|p| !p.is_empty()).is_none() {
        return Err(InternalErr::ChapterNoPage(chapter_id));
    }

    Ok((StatusCode::OK, Json(pages)).into_response())
}
