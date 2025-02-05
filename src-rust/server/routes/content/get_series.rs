use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use super::{is_jxl, parse_tags, structs::BaseTitleResponse};
use crate::{
    routes::errors::InternalErr,
    structs::id::UserID,
    utils::{app_state::AppState, constants::GET_SERIES_PATH},
};

#[skip_serializing_none]
#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct InnerSeriesResponseChapter {
    pub id: String,
    pub number: i32,
    pub description: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct SeriesResponse {
    #[serde(flatten)]
    base: BaseTitleResponse,

    chapters: Option<Vec<InnerSeriesResponseChapter>>,
}

/// Series info & chapters
#[utoipa::path(get, path = GET_SERIES_PATH, responses(
    (status = 200, description = "Fetch series success", body = SeriesResponse),
    (status = 303, description = "Title is an oneshot, redirect to the correct endpoint"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 404, description = "No series found for the given id"),
    (status = 500, description = "Internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_series(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<i64>,
    Extension(user_id): Extension<UserID>,
) -> Result<Response, InternalErr> {
    let Some(r) = sqlx::query!(
        r#"SELECT t.title AS title,
            c.name AS "category?",
            t.author AS author,
            t.description AS "description?",
            t.release AS release,
            t.is_series AS is_series,
            t.cover_path AS cover_path,
            t.cover_blurhash AS cover_blurhash,
            t.cover_width AS cover_width,
            t.cover_height AS cover_height,
            t.date_updated AS date_updated,
            ARRAY_AGG(
                json_build_array(tg.id, tg.name)
            ) FILTER (
                WHERE tg.name IS NOT NULL
            ) AS "tags",
            ARRAY_AGG(
                json_build_object(
                    'id',
                    ch.id,
                    'number',
                    ch.number,
                    'description',
                    ch.description
                )
            ) AS "chapters",
            (
                SELECT COUNT(*)
                FROM favorites
                WHERE title_id = $1
            ) AS "favorites_count!",
            (
                SELECT COUNT(*)
                FROM bookmarks
                WHERE title_id = $1
            ) AS "bookmarks_count!",
            EXISTS(
                SELECT 1
                FROM favorites
                WHERE title_id = $1
                    AND user_id = $2
            ) AS "is_favorite!",
            EXISTS(
                SELECT 1
                FROM bookmarks
                WHERE title_id = $1
                    AND user_id = $2
            ) AS "is_bookmark!",
            pr.page AS "page_read?"
        FROM titles t
            LEFT JOIN categories c ON c.id = t.category_id
            LEFT JOIN titles_tags tt ON tt.title_id = t.id
            LEFT JOIN tags tg ON tg.id = tt.tag_id
            LEFT JOIN progresses pr ON (
                pr.title_id = t.id
                AND pr.user_id = $2
            )
            LEFT JOIN chapters ch ON ch.title_id = t.id
        WHERE t.id = $1
        GROUP BY t.id,
            c.name,
            pr.page"#,
        title_id,
        user_id.as_ref()
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    if !r.is_series {
        return Ok((
            StatusCode::SEE_OTHER,
            Redirect::to(format!("/api/content/oneshot/{title_id}").as_str()),
        )
            .into_response());
    }

    let body = SeriesResponse {
        base: BaseTitleResponse {
            id: title_id.to_string(),
            title: r.title,
            category_id: r.category,
            author: r.author,
            description: r.description,
            release: r.release.map(|d| d.format("%Y-%m-%d").to_string()),

            cover_blurhash: r.cover_blurhash,
            cover_width: r.cover_width,
            cover_height: r.cover_height,
            tags: r.tags.and_then(parse_tags),
            cover_jxl: r.cover_path.map(is_jxl),

            date_updated: r.date_updated.map(|d| d.to_rfc3339()),
            favorites: (r.favorites_count != 0).then_some(r.favorites_count),
            bookmarks: (r.bookmarks_count != 0).then_some(r.bookmarks_count),

            is_favorite: r.is_favorite,
            is_bookmark: r.is_bookmark,
            page_read: r.page_read.filter(|i| *i != 0),
        },
        chapters: r.chapters.map(|rs| {
            rs.into_iter()
                .filter_map(|r| {
                    let chapter = r.as_object()?;

                    Some(InnerSeriesResponseChapter {
                        id: chapter.get("id")?.as_i64()?.to_string(),
                        number: chapter.get("number")?.as_i64()? as i32,
                        description: chapter
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                    })
                })
                .collect()
        }),
    };

    if body.chapters.as_ref().filter(|c| !c.is_empty()).is_none() {
        let e = InternalErr::NoChapter(title_id);
        tracing::error!("{e}");
        return Err(e);
    }

    Ok((StatusCode::OK, Json(body)).into_response())
}
