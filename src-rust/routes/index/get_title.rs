use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use crate::{types::custom_id::CustomID, AppError, AppState};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct ResponsePage {
    pub id: String,
    pub format: String,
    pub description: Option<String>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct TitleResponseBody {
    pub category_id: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,

    pub cover_blurhash: Option<String>,
    pub cover_width: Option<i32>,
    pub cover_height: Option<i32>,

    pub tag_ids: Vec<String>,
    pub pages: Vec<ResponsePage>,
    pub favorites: i64,
    pub bookmarks: i64,
    pub is_favorite: bool,
    pub is_bookmark: bool,
    pub page_read: Option<i32>,
    pub date_added: String,
    pub date_updated: Option<String>,
}

/// Get everything about a title.
#[utoipa::path(get, path = "/api/index/title/{title_id}", responses(
    (status = 200, description = "Fetch title successful", body = TitleResponseBody),
    (status = 204, description = "No title found for the given id", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn get_title(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<Response, AppError> {
    let title_id = match CustomID::from(title_id) {
        Ok(id) => id,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, format!("{e:#}")).into_response()),
    };

    let title_record = match sqlx::query!("SELECT * FROM titles WHERE id = $1", title_id.as_str())
        .fetch_optional(&app_state.pool)
        .await
        .context("can't find title")?
    {
        Some(result) => result,
        None => return Ok((StatusCode::NOT_FOUND, "no title found").into_response()),
    };

    let pages = sqlx::query!(
        "SELECT * FROM pages WHERE title_id = $1 ORDER BY path ASC",
        title_id.as_str()
    )
    .fetch_all(&app_state.pool)
    .await
    .context("can't find pages")?;

    // place the cover.path at the front of the Vec<pages::Model>
    // and convert it to Vec<ResponsePage>
    let pages = pages
        .into_iter()
        .fold(Vec::new(), |mut list, page_model| {
            match &title_record.cover_path {
                Some(cover_path) => match page_model.path.as_str() == cover_path {
                    true => list.insert(0, page_model),
                    false => list.push(page_model),
                },
                None => list.push(page_model),
            }
            list
        })
        .into_iter()
        .map(|page| ResponsePage {
            id: page.id.to_string(),
            format: PathBuf::from(page.path)
                .extension()
                .map(|s| s.to_str().unwrap_or(""))
                .unwrap_or("")
                .to_ascii_lowercase(),
            description: page.description,
        })
        .collect::<Vec<_>>();

    let is_favorite = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM favorites WHERE user_id = $1 AND title_id = $2) AS "exists!""#,
        &user_id,
        title_record.id.as_str()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't check if user is favorite")?
    .exists;

    let is_bookmark = sqlx::query!(
        r#"SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND title_id = $2) AS "exists!""#,
        &user_id,
        title_record.id.as_str()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't check if user is bookmark")?
    .exists;

    let page_read = sqlx::query!(
        r#"SELECT page FROM progresses WHERE user_id = $1 AND title_id = $2"#,
        &user_id,
        title_record.id.as_str()
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't check if user is bookmark")?
    .map(|p| p.page);

    let favorites = sqlx::query!(
        r#"SELECT COUNT(*) AS "count!" FROM favorites WHERE title_id = $1"#,
        title_record.id.as_str()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't check if user is bookmark")?
    .count;

    let bookmarks = sqlx::query!(
        r#"SELECT COUNT(*) AS "count!" FROM bookmarks WHERE title_id = $1"#,
        title_record.id.as_str()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't check if user is bookmark")?
    .count;

    let tag_ids = sqlx::query!(
        r#"SELECT tag_id FROM titles_tags WHERE title_id = $1"#,
        title_record.id.as_str()
    )
    .fetch_all(&app_state.pool)
    .await
    .context("can't check if user is bookmark")?
    .into_iter()
    .map(|tag| tag.tag_id)
    .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(TitleResponseBody {
            category_id: title_record.category_id.map(|id| id.to_string()),
            title: title_record.title,
            author: title_record.author,
            description: title_record.description,
            release_date: title_record
                .release
                .map(|d| d.format("%Y-%m-%dT00:00:00Z").to_string()),
            cover_blurhash: title_record.cover_blurhash,
            cover_width: title_record.cover_width,
            cover_height: title_record.cover_height,
            tag_ids,
            pages,
            favorites,
            bookmarks,
            is_favorite,
            is_bookmark,
            page_read,
            date_added: title_record.date_added.to_rfc3339(),
            date_updated: title_record.date_updated.map(|d| d.to_rfc3339()),
        }),
    )
        .into_response())
}
