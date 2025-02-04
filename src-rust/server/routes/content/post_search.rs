use std::sync::Arc;

use crate::{routes::errors::InternalErr, structs::id::UserID, utils::app_state::AppState};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::structs::BaseTitleResponse;

#[derive(Debug, ToSchema, Serialize, Deserialize, Default)]
pub enum InnerSearchRequestOrderBy {
    #[serde(rename = "title")]
    #[default]
    Title,
    #[serde(rename = "release_date")]
    ReleaseDate,
    #[serde(rename = "update_date")]
    UpdateDate,
}

impl AsRef<str> for InnerSearchRequestOrderBy {
    fn as_ref(&self) -> &str {
        match self {
            InnerSearchRequestOrderBy::Title => "title",
            InnerSearchRequestOrderBy::ReleaseDate => "release_date",
            InnerSearchRequestOrderBy::UpdateDate => "update_date",
        }
    }
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct SearchRequest {
    pub term: Option<String>,

    pub category_ids: Option<Vec<String>>,
    pub tag_ids: Option<Vec<String>>,
    pub release_year: Option<i64>,

    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub order_by: Option<InnerSearchRequestOrderBy>,
    pub is_ascending: Option<bool>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct InnerTitleSearchResponse {
    pub is_series: bool,

    #[serde(flatten)]
    pub base: BaseTitleResponse,
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct SearchResponse {
    pub data: Option<Vec<InnerTitleSearchResponse>>,

    pub offset: i64,
    pub limit: i64,
}

/// Search title
#[utoipa::path(post, path = "api/content/search", responses(
    (status = 200, description = "Search success", body = SearchResponse),
    (status = 204, description = "Search success, but none were found", body = SearchResponse),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn post_search(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Json(request_body): Json<SearchRequest>,
) -> Result<Response, InternalErr> {
    let limit = request_body.limit.unwrap_or(10);
    let offset = request_body.offset.unwrap_or(0);
    let order_by = request_body.order_by.unwrap_or_default();
    let is_ascending = request_body.is_ascending.unwrap_or(true);

    let title_ids: Option<Vec<i64>> = None;
    let category_ids = request_body
        .category_ids
        .map(|ids| {
            ids.into_iter()
                .filter_map(|id| id.parse::<i64>().ok())
                .collect()
        })
        .filter(|ids: &Vec<i64>| !ids.is_empty());
    let tag_ids = request_body
        .tag_ids
        .map(|ids| {
            ids.into_iter()
                .filter_map(|id| id.parse::<i64>().ok())
                .collect()
        })
        .filter(|ids: &Vec<i64>| !ids.is_empty());
    let release_year = request_body.release_year.map(|year| year as i32);

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
        WHERE (
                -- title ids
                $2::bigint [] IS NULL
                OR t.id IN (
                    SELECT title_id
                    FROM UNNEST($2::bigint []) AS t(title_id)
                )
            )
            AND (
                -- category ids
                $3::bigint [] IS NULL
                OR c.id IN (
                    SELECT category_id
                    FROM UNNEST($3::bigint []) AS c(category_id)
                )
            )
            AND (
                -- tag ids
                $4::bigint [] IS NULL
                OR tg.id IN (
                    SELECT tag_id
                    FROM UNNEST($4::bigint []) AS tg(tag_id)
                )
            )
            AND (
                -- release year
                $5::int IS NULL
                OR EXTRACT(
                    YEAR
                    FROM t.release
                ) = $5
            )
        GROUP BY t.id,
            c.id,
            pr.page
        ORDER BY CASE
                WHEN $6 THEN $7
            END ASC,
            CASE
                WHEN NOT $6 THEN $7
            END DESC
        LIMIT $8 OFFSET $9"#,
        user_id.as_ref(),
        title_ids.as_deref(),
        category_ids.as_deref(),
        tag_ids.as_deref(),
        release_year,
        is_ascending,
        order_by.as_ref(),
        limit,
        offset,
    )
    .fetch_all(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        InternalErr::DB(e)
    })?
    .into_iter()
    .map(|r| InnerTitleSearchResponse {
        is_series: r.is_series,
        base: BaseTitleResponse {
            id: r.id.to_string(),
            title: r.title,
            author: r.author,
            category_id: r.category_id.map(|i| i.to_string()),
            description: r.description,
            release: r.release.map(|d| d.format("%Y-%m-%d").to_string()),

            cover_blurhash: r.cover_blurhash,
            cover_width: r.cover_width,
            cover_height: r.cover_height,
            cover_jxl: r.cover_path.map(|p| {
                std::path::Path::new(&p)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
            }),
            date_updated: r.date_updated.map(|d| d.to_rfc3339()),
            tags: r.tags.map(|mut tags| {
                tags.iter_mut()
                    .filter_map(|tag| {
                        let mut pair = tag.as_array()?.iter();
                        Some((
                            pair.next()?.as_i64()?.to_string(),
                            pair.next()?.as_str()?.to_string(),
                        ))
                    })
                    .collect()
            }),
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
        },
    })
    .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(SearchResponse {
            data: if data.is_empty() { None } else { Some(data) },
            offset,
            limit,
        }),
    )
        .into_response())
}
