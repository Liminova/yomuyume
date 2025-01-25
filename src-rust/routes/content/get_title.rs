use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};

use crate::{
    routes::TitleResponseBody,
    traits::tags_split::TagsSplit,
    types::UserID,
    utils::{app_error::AppError, app_state::AppState},
};

/// get title
///
/// with all the information
#[utoipa::path(get, path = "/api/content/title/{title_id}", responses(
    (status = 200, description = "fetch title successful", body = TitleResponseBody),
    (status = 401, description = "unauthorized", body = String),
    (status = 404, description = "no title found for the given id"),
    (status = 500, description = "internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_title(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<i64>,
    Extension(user_id): Extension<UserID>,
) -> Result<Response, AppError> {
    let Some(body) = sqlx::query!(
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
            ARRAY_AGG(DISTINCT CONCAT(tg.id, '-', tg.name)) FILTER (
                WHERE tg.name IS NOT NULL
            ) AS "tags",
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
            LEFT JOIN oneshots_pages p ON p.title_id = t.id
            LEFT JOIN titles_tags tt ON tt.title_id = t.id
            LEFT JOIN tags tg ON tg.id = tt.tag_id
            LEFT JOIN progresses pr ON (
                pr.title_id = t.id
                AND pr.user_id = $2
            )
        WHERE t.id = $1
        GROUP BY t.id,
            c.name,
            pr.page"#,
        title_id,
        user_id.as_ref()
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query title")?
    .map(|r| TitleResponseBody {
        id: title_id.to_string(),
        title: r.title,
        category_id: r.category,
        author: r.author,
        description: r.description,
        release: r.release.map(|d| d.format("%Y-%m-%d").to_string()),
        is_series: r.is_series,

        cover_blurhash: r.cover_blurhash,
        cover_width: r.cover_width,
        cover_height: r.cover_height,
        tags: r.tags.tags_split(),
        cover_jxl: r.cover_path.map(|path| {
            std::path::Path::new(&path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
        }),

        date_updated: r.date_updated.map(|d| d.to_rfc3339()),

        favorites: if r.favorites_count != 0 {
            Some(r.favorites_count)
        } else {
            None
        },
        bookmarks: if r.bookmarks_count != 0 {
            Some(r.favorites_count)
        } else {
            None
        },

        is_favorite: r.is_favorite,
        is_bookmark: r.is_bookmark,
        page_read: r.page_read.filter(|i| *i != 0),
    }) else {
        return Ok((StatusCode::NOT_FOUND).into_response());
    };

    Ok((StatusCode::OK, Json(body)).into_response())
}
