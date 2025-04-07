use std::sync::Arc;

use crate::{
    routes::errors::InternalErr,
    structs::ids::UserID,
    utils::{app_state::AppState, constants::SEARCH_PATH},
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use axum_extra::extract::Query;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{is_jxl, parse_tags, structs::BaseTitleResponse};

#[derive(Debug, ToSchema, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InnerSearchRequestOrderBy {
    #[default]
    Title,
    Release,
    DateUpdated,
    Author,
    ProgressLastReadAt,
}

impl AsRef<str> for InnerSearchRequestOrderBy {
    fn as_ref(&self) -> &str {
        use InnerSearchRequestOrderBy as O;
        match self {
            O::Title => "title",
            O::Release => "release",
            O::DateUpdated => "date_updated",
            O::Author => "author",
            O::ProgressLastReadAt => "progress_last_read_at",
        }
    }
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct SearchQuery {
    pub term: Option<String>,

    pub category_ids: Option<Vec<String>>,
    pub tag_ids: Option<Vec<String>>,
    pub release_year: Option<i64>,

    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub order_by: Option<InnerSearchRequestOrderBy>,
    pub is_ascending: Option<bool>,

    pub is_bookmarked: Option<bool>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: Vec<BaseTitleResponse>,

    pub offset: i64,
    pub limit: i64,
}

/// Search title
#[utoipa::path(
    get,
    path = SEARCH_PATH,
    responses(
        (status = 200, description = "Search success", body = SearchResponse),
        (status = 204, description = "Search success, but none were found", body = SearchResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    params(
        ("limit" = Option<i64>, Query, description = "Limit"),
        ("offset" = Option<i64>, Query, description = "Offset"),
        ("order_by" = Option<InnerSearchRequestOrderBy>, Query, description = "Order by"),
        ("is_ascending" = Option<bool>, Query, description = "Is ascending"),
        ("term" = Option<String>, Query, description = "Term"),
        ("category_ids" = Option<Vec<String>>, Query, description = "Category ids"),
        ("tag_ids" = Option<Vec<String>>, Query, description = "Tag ids"),
        ("release_year" = Option<i64>, Query, description = "Release year"),
    ),
    security(("session-id" = [], "session-secret" = [])))
]
pub async fn get_search(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Query(query): Query<SearchQuery>,
) -> Result<Response, InternalErr> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    let order_by = query.order_by.unwrap_or_default();

    let category_ids = query
        .category_ids
        .map(|ids| {
            ids.into_iter()
                .filter_map(|id| id.parse::<i64>().ok())
                .collect()
        })
        .filter(|ids: &Vec<i64>| !ids.is_empty());
    let tag_ids = query
        .tag_ids
        .map(|ids| {
            ids.into_iter()
                .filter_map(|id| id.parse::<i64>().ok())
                .collect()
        })
        .filter(|ids: &Vec<i64>| !ids.is_empty());

    let data = sqlx::query!(
        r#"SELECT t.id AS id,
            t.title AS title,
            c.name AS "category?",
            c.id AS "category_id?",
            t.author AS author,
            t.description AS "description?",
            t.release AS release,
            t.cover_path AS cover_path,
            t.cover_blurhash AS cover_blurhash,
            t.cover_width AS cover_width,
            t.cover_height AS cover_height,
            t.date_updated AS date_updated,
            t.path AS path,
            ARRAY_AGG(
                json_build_array(tg.id, tg.name)
            ) FILTER (
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
            pr.chapter_id AS "progress_chapter_id?",
            pr.page_id AS "progress_page_id?",
            pr.percent AS "progress_percent?",
            pr.last_read_at AS "progress_last_read_at?"
        FROM titles t
            LEFT JOIN categories c ON c.id = t.category_id
            LEFT JOIN chapters ch ON ch.title_id = t.id
            LEFT JOIN pages p ON p.chapter_id = ch.id
            LEFT JOIN titles_tags tt ON tt.title_id = t.id
            LEFT JOIN tags tg ON tg.id = tt.tag_id
            LEFT JOIN progresses pr ON (
                pr.title_id = t.id
                AND pr.user_id = $1
            )
        WHERE (
                -- category ids
                $2::bigint [] IS NULL
                OR c.id IN (
                    SELECT category_id
                    FROM UNNEST($2::bigint []) AS c(category_id)
                )
            )
            AND (
                -- tag ids
                $3::bigint [] IS NULL
                OR tg.id IN (
                    SELECT tag_id
                    FROM UNNEST($3::bigint []) AS tg(tag_id)
                )
            )
            AND (
                -- release year
                $4::int IS NULL
                OR EXTRACT(
                    YEAR
                    FROM t.release
                ) = $4
            )
            AND (
                -- is bookmarked
                $5::bool IS NULL
                OR EXISTS(
                    SELECT 1
                    FROM bookmarks
                    WHERE title_id = t.id
                        AND user_id = $1
                ) = $5
            )
            AND (
                -- is favorite
                $6::bool IS NULL
                OR EXISTS(
                    SELECT 1
                    FROM favorites
                    WHERE title_id = t.id
                        AND user_id = $1
                ) = $6
            )
        GROUP BY t.id,
            c.id,
            pr.title_id,
            pr.user_id
        ORDER BY CASE
                WHEN $7 THEN $8
            END ASC,
            CASE
                WHEN NOT $7 THEN $8
            END DESC
        LIMIT $9 OFFSET $10"#,
        user_id.as_ref(),
        category_ids.as_deref(),
        tag_ids.as_deref(),
        query.release_year.map(|year| year as i32),
        query.is_bookmarked,
        query.is_favorite,
        query.is_ascending.unwrap_or(true),
        order_by.as_ref(),
        limit,
        offset,
    )
    .fetch_all(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?
    .into_iter()
    .map(|r| BaseTitleResponse {
        id: r.id.to_string(),
        title: r.title,
        author: r.author,
        category_id: r.category_id.map(|i| i.to_string()),
        description: r.description,
        release: r.release.map(|d| d.format("%Y-%m-%d").to_string()),

        cover_blurhash: r.cover_blurhash,
        cover_width: r.cover_width,
        cover_height: r.cover_height,
        cover_jxl: r.cover_path.map(is_jxl),
        date_updated: r.date_updated.map(|d| d.to_rfc3339()),
        tags: r.tags.and_then(parse_tags),
        favorites: (r.favorites_count != 0).then_some(r.favorites_count),
        bookmarks: (r.bookmarks_count != 0).then_some(r.bookmarks_count),

        is_favorite: r.is_favorite,
        is_bookmark: r.is_bookmark,

        progress_page_id: r.progress_page_id.map(|id| id.to_string()),
        progress_chapter_id: r.progress_chapter_id.map(|id| id.to_string()),
        progress_percent: r.progress_percent,
        progress_last_read_at: r.progress_last_read_at.map(|d| d.to_rfc3339()),
    })
    .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(SearchResponse {
            data,
            offset,
            limit,
        }),
    )
        .into_response())
}
