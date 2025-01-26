use std::sync::Arc;

use crate::{
    routes::TitleResponseBody,
    traits::tags_split::TagsSplit,
    types::UserID,
    utils::{app_error::AppError, app_state::AppState},
};

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize, Deserialize, Default)]
pub enum OrderBy {
    #[serde(rename = "title")]
    #[default]
    Title,
    #[serde(rename = "release_date")]
    ReleaseDate,
    #[serde(rename = "update_date")]
    UpdateDate,
}

impl AsRef<str> for OrderBy {
    fn as_ref(&self) -> &str {
        match self {
            OrderBy::Title => "title",
            OrderBy::ReleaseDate => "release_date",
            OrderBy::UpdateDate => "update_date",
        }
    }
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct FilterRequestBody {
    pub keywords: String,

    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub order_by: Option<OrderBy>,
    pub is_ascending: Option<bool>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct FilterResponseBody {
    pub data: Vec<TitleResponseBody>,

    pub offset: i64,
    pub limit: i64,
}

/// quick search for titles by keywords
///
/// and also sorting them by various options
#[utoipa::path(post, path = "api/content/search", responses(
    (status = 200, description = "fetch all items successful", body = FilterResponseBody),
    (status = 204, description = "fetch all items successful, but none were found", body = FilterResponseBody),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn post_search(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Json(request_body): Json<FilterRequestBody>,
) -> Result<Response, AppError> {
    let keywords = request_body
        .keywords
        .split_whitespace()
        .map(|s| {
            s.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>()
        .join("|");

    let limit = request_body.limit.unwrap_or(10);
    let offset = request_body.offset.unwrap_or(0);
    let order_by = request_body
        .order_by
        .map_or("title", |order_by| match order_by {
            OrderBy::ID => "id",
            OrderBy::Title => "title",
            OrderBy::ReleaseDate => "release",
            OrderBy::UpdateDate => "date_updated",
        });
    let is_ascending = request_body.is_ascending.unwrap_or(true);

    let data = sqlx::query!(
        r#"SELECT t.id AS id,
            t.title AS title,
            c.name AS "category?",
            c.id AS "category_id?",
            t.author AS author,
            t.description AS "description?",
            t.release AS release,
            t.is_series AS is_series,
            t.cover_path AS cover_path,
            t.cover_blurhash AS cover_blurhash,
            t.cover_width AS cover_width,
            t.cover_height AS cover_height,
            t.date_updated AS date_updated,
            t.path AS path,
            ARRAY_AGG(DISTINCT CONCAT(tg.id, '-', tg.name)) FILTER (
                WHERE tg.name IS NOT NULL
            ) AS "tags",
            (
                SELECT COUNT(*)
                FROM favorites
                WHERE title_id = t.id
            ) AS "favorites_count!",
            (
                SELECT COUNT(*)
                FROM bookmarks
                WHERE title_id = t.id
            ) AS "bookmarks_count!",
            EXISTS(
                SELECT 1
                FROM favorites
                WHERE title_id = t.id
                    AND user_id = $1
            ) AS "is_favorite!",
            EXISTS(
                SELECT 1
                FROM bookmarks
                WHERE title_id = t.id
                    AND user_id = $1
            ) AS "is_bookmark!",
            pr.page AS "page_read?"
        FROM titles t
            LEFT JOIN categories c ON c.id = t.category_id
            LEFT JOIN oneshots_pages p ON p.title_id = t.id
            LEFT JOIN titles_tags tt ON tt.title_id = t.id
            LEFT JOIN tags tg ON tg.id = tt.tag_id
            LEFT JOIN progresses pr ON (
                pr.title_id = t.id
                AND pr.user_id = $1
            )
        WHERE t.path ~* $2
            OR t.title ~* $2
            OR t.author ~* $2
            OR t.description ~* $2
        GROUP BY t.id,
            c.id,
            pr.page
        ORDER BY CASE
                WHEN $3 THEN $4
            END ASC,
            CASE
                WHEN NOT $3 THEN $4
            END DESC
        LIMIT $5 OFFSET $6"#,
        user_id.as_ref(),
        keywords,
        is_ascending,
        order_by,
        limit,
        offset,
    )
    .fetch_all(&app_state.pool)
    .await
    .context("can't query titles")?
    .into_iter()
    .map(|r| TitleResponseBody {
        id: r.id.to_string(),
        title: r.title,
        author: r.author,
        category_id: r.category_id.map(|i| i.to_string()),
        description: r.description,
        release: r.release.map(|d| d.format("%Y-%m-%d").to_string()),
        is_series: r.is_series,

        cover_blurhash: r.cover_blurhash,
        cover_width: r.cover_width,
        cover_height: r.cover_height,
        cover_jxl: r.cover_path.map(|p| {
            std::path::Path::new(&p)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
        }),
        date_updated: r.date_updated.map(|d| d.to_rfc3339()),
        tags: r.tags.tags_split(),
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
    })
    .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(FilterResponseBody {
            data,
            offset,
            limit,
        }),
    )
        .into_response())
}
