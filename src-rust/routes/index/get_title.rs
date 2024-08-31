use std::{path::PathBuf, sync::Arc};

use crate::{models::prelude::*, routes::calculate_dimension, AppError, AppState};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
#[skip_serializing_none]
pub struct ResponsePage {
    pub id: String,
    pub format: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
#[skip_serializing_none]
pub struct ResponseCover {
    pub blurhash: Option<String>,
    pub width: Option<u8>,
    pub height: Option<u8>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
#[skip_serializing_none]
pub struct TitleResponseBody {
    pub category_id: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub cover: ResponseCover,
    pub tag_ids: Vec<u32>,
    pub pages: Vec<ResponsePage>,
    pub favorites: Option<i64>,
    pub bookmarks: Option<i64>,
    pub is_favorite: Option<bool>,
    pub is_bookmark: Option<bool>,
    pub page_read: Option<i64>,
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
    Extension(user): Extension<users::Model>,
) -> Result<Response, AppError> {
    let title_id = match CustomID::from(title_id) {
        Ok(id) => id,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e).into_response()),
    };

    let title = match Titles::find_by_id(title_id)
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find title: {}", e)))?
    {
        Some(title) => title,
        None => return Ok((StatusCode::NOT_FOUND, "no title found").into_response()),
    };

    let pages = Pages::find()
        .filter(pages::Column::TitleId.eq(&title.id))
        .order_by_asc(pages::Column::Path)
        .all(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find pages: {}", e)))?;

    // place the cover.path at the front of the Vec<pages::Model>
    // and convert it to Vec<ResponsePage>
    let pages = pages
        .into_iter()
        .fold(Vec::new(), |mut list, page_model| {
            match &title.cover_path {
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

    let is_favorite = Favorites::find()
        .filter(
            Condition::all()
                .add(favorites::Column::UserId.eq(&user.id))
                .add(favorites::Column::TitleId.eq(&title.id)),
        )
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find favorite: {}", e)))?
        .map(|_| true);

    let is_bookmark = Bookmarks::find()
        .filter(
            Condition::all()
                .add(bookmarks::Column::UserId.eq(&user.id))
                .add(bookmarks::Column::TitleId.eq(&title.id)),
        )
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find bookmark: {}", e)))?
        .map(|_| true);

    let page_read = Progresses::find()
        .filter(
            Condition::all()
                .add(progresses::Column::UserId.eq(&user.id))
                .add(progresses::Column::TitleId.eq(&title.id)),
        )
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find progress: {}", e)))?
        .map(|p| p.page);

    let favorites = match Favorites::find()
        .filter(favorites::Column::TitleId.eq(&title.id))
        .count(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find favorites: {}", e)))?
    {
        0 => None,
        n => Some(n as i64),
    };

    let bookmarks = match Bookmarks::find()
        .filter(bookmarks::Column::TitleId.eq(&title.id))
        .count(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find bookmarks: {}", e)))?
    {
        0 => None,
        n => Some(n as i64),
    };

    let tag_ids = TitlesTags::find()
        .filter(titles_tags::Column::TitleId.eq(&title.id))
        .all(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find tags: {}", e)))?
        .iter()
        .map(|tag| tag.tag_id)
        .collect::<Vec<_>>();

    let (width, height) = calculate_dimension(&app_state.config, cover.ratio);

    Ok((
        StatusCode::OK,
        Json(TitleResponseBody {
            category_id: title.category_id.to_string(),
            title: title.title,
            author: title.author,
            description: title.description,
            release_date: title.release,
            cover: ResponseCover {
                blurhash: cover.blurhash,
                width,
                height,
                format: PathBuf::from(cover.path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase(),
            },
            tag_ids,
            pages,
            favorites,
            bookmarks,
            is_favorite,
            is_bookmark,
            page_read,
            date_added: title.date_added,
            date_updated: title.date_updated,
        }),
    )
        .into_response())
}
