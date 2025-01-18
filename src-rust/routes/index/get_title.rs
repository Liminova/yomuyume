use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    types::UserID,
    utils::{app_error::AppError, app_state::AppState},
};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct TitleTagResponse {
    pub id: String,
    pub name: String,
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct TitleResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    pub is_series: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_blurhash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_width: Option<i32>,
    pub cover_height: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_updated: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TitleTagResponse>,
    pub favorites: i64,
    pub bookmarks: i64,

    pub is_favorite: bool,
    pub is_bookmark: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_read: Option<i32>,
}

/// get title
///
/// with all the information
#[utoipa::path(get, path = "/api/index/title/{title_id}", responses(
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
    let Some(title_record) = sqlx::query!(
        r#"
        SELECT
            t.title AS title,

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

            ARRAY_AGG(DISTINCT CONCAT(tg.name, '-', tg.id)) AS "tags",

            COALESCE(fav.count, 0) AS "favorites_count!",
            COALESCE(bkm.count, 0) AS "bookmarks_count!",
            CASE WHEN fav_user.count = 1 THEN TRUE ELSE FALSE END AS "is_favorite!",
            CASE WHEN bkm_user.count = 1 THEN TRUE ELSE FALSE END AS "is_bookmark!",
            pr.page AS "page_read?"

        FROM titles t
            LEFT JOIN categories c ON c.id = t.category_id
            LEFT JOIN oneshots_pages p ON p.title_id = t.id
            LEFT JOIN titles_tags tt ON tt.title_id = t.id
            LEFT JOIN tags tg ON tg.id = tt.tag_id
            LEFT JOIN progresses pr ON (pr.title_id = t.id AND pr.user_id = $2)

            -- count favorites and bookmarks
            LEFT JOIN (
                SELECT title_id, COUNT(*) AS count
                FROM favorites WHERE title_id = $1
                GROUP BY title_id
            ) fav ON fav.title_id = t.id
            LEFT JOIN (
                SELECT title_id, COUNT(*) AS count
                FROM bookmarks WHERE title_id = $1
                GROUP BY title_id
            ) bkm ON bkm.title_id = t.id

            -- count favorites and bookmarks by user (either 1 or NULL)
            LEFT JOIN (
                SELECT title_id, COUNT(*) AS count
                FROM favorites
                WHERE title_id = $1 AND user_id = $2
                GROUP BY title_id
            ) fav_user ON fav_user.title_id = t.id
            LEFT JOIN (
                SELECT title_id, COUNT(*) AS count
                FROM bookmarks
                WHERE title_id = $1 AND user_id = $2
                GROUP BY title_id
            ) bkm_user ON bkm_user.title_id = t.id

        WHERE t.id = $1
        GROUP BY
            t.id, c.name, t.description,
            fav.count, bkm.count, fav_user.count, bkm_user.count, pr.page
        "#,
        title_id,
        user_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query title")?
    else {
        return Ok((StatusCode::NOT_FOUND).into_response());
    };

    let body = TitleResponseBody {
        title: title_record.title,
        category_id: title_record.category,
        author: title_record.author,
        description: title_record.description,
        release: title_record
            .release
            .map(|d| d.format("%Y-%m-%d").to_string()),
        is_series: title_record.is_series,

        cover_blurhash: title_record.cover_blurhash,
        cover_width: title_record.cover_width,
        cover_height: title_record.cover_height,
        tags: title_record.tags.map_or_else(Vec::new, |tags| {
            tags.into_iter()
                .map(|s| {
                    let parts = s.split('-').collect::<Vec<_>>();
                    TitleTagResponse {
                        id: parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
                        name: parts.first().map(|s| s.to_string()).unwrap_or_default(),
                    }
                })
                .collect()
        }),

        date_updated: title_record.date_updated.map(|d| d.to_rfc3339()),

        favorites: title_record.favorites_count,
        bookmarks: title_record.bookmarks_count,

        is_favorite: title_record.is_favorite,
        is_bookmark: title_record.is_bookmark,
        page_read: title_record.page_read,
    };

    Ok((StatusCode::OK, Json(body)).into_response())
}
