use crate::{
    models::prelude::*,
    routes::{calculate_dimension, ErrRsp},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use sea_orm::*;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{path::PathBuf, sync::Arc};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema, Debug)]
#[skip_serializing_none]
pub struct ResponsePage {
    pub id: String,
    pub format: String,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ResponseThumbnail {
    pub blurhash: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Serialize, ToSchema)]
#[skip_serializing_none]
pub struct TitleResponseBody {
    pub category_id: String,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub thumbnail: ResponseThumbnail,
    pub tag_ids: Vec<u32>,
    pub pages: Vec<ResponsePage>,
    pub favorites: Option<i64>,
    pub bookmarks: Option<i64>,
    pub is_favorite: Option<bool>,
    pub is_bookmark: Option<bool>,
    pub page_read: Option<i64>,
    pub date_added: String,
    pub date_updated: String,
}

/// Get everything about a title.
#[utoipa::path(get, path = "/api/index/title/{title_id}", responses(
    (status = 200, description = "Fetch title successful", body = TitleResponseBody),
    (status = 204, description = "No title found for the given id", body = TitleResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn get_title(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<Uuid>,
    Extension(user): Extension<users::Model>,
) -> Result<impl IntoResponse, ErrRsp> {
    let title = Titles::find_by_id(title_id)
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[1] DB error getting title: {}", e)))?
        .ok_or_else(|| ErrRsp::new(StatusCode::NO_CONTENT, "No title found."))?;

    let thumbnail = Thumbnails::find_by_id(&title.id)
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[2] DB error getting thumbnail: {}", e)))?
        .ok_or_else(|| ErrRsp::new(StatusCode::NO_CONTENT, "No thumbnail found."))?;

    let pages = Pages::find()
        .filter(pages::Column::TitleId.eq(&title.id))
        .order_by_asc(pages::Column::Path)
        .all(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[3] DB error getting pages: {}", e)))?;

    // place the thumbnail.path at the front of the Vec<pages::Model>
    // and convert it to Vec<ResponsePage>
    let pages = pages
        .into_iter()
        .fold(Vec::new(), |mut list, page_model| {
            if page_model.path == thumbnail.path {
                list.insert(0, page_model);
            } else {
                list.push(page_model);
            }
            list
        })
        .into_iter()
        .map(|page| ResponsePage {
            id: page.id,
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
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[4] DB error getting favorite: {}", e)))?
        .map(|_| true);

    let is_bookmark = Bookmarks::find()
        .filter(
            Condition::all()
                .add(bookmarks::Column::UserId.eq(&user.id))
                .add(bookmarks::Column::TitleId.eq(&title.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[5] DB error getting bookmark: {}", e)))?
        .map(|_| true);

    let page_read = Progresses::find()
        .filter(
            Condition::all()
                .add(progresses::Column::UserId.eq(&user.id))
                .add(progresses::Column::TitleId.eq(&title.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[6] DB error getting progress: {}", e)))?
        .map(|p| p.page);

    let favorites = match Favorites::find()
        .filter(favorites::Column::TitleId.eq(&title.id))
        .count(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[7] DB error getting favorites: {}", e)))?
    {
        0 => None,
        n => Some(n as i64),
    };

    let bookmarks = match Bookmarks::find()
        .filter(bookmarks::Column::TitleId.eq(&title.id))
        .count(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[7] DB error getting bookmarks: {}", e)))?
    {
        0 => None,
        n => Some(n as i64),
    };

    let tag_ids = TitlesTags::find()
        .filter(titles_tags::Column::TitleId.eq(&title.id))
        .all(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("[8] DB error getting tags: {}", e)))?
        .iter()
        .map(|tag| tag.tag_id)
        .collect::<Vec<_>>();

    let (width, height) = calculate_dimension(thumbnail.ratio);

    Ok((
        StatusCode::OK,
        Json(TitleResponseBody {
            category_id: title.category_id,
            title: title.title,
            author: title.author,
            description: title.description,
            release_date: title.release,
            thumbnail: ResponseThumbnail {
                blurhash: thumbnail.blurhash,
                width,
                height,
                format: PathBuf::from(thumbnail.path)
                    .extension()
                    .map(|s| s.to_str().unwrap_or(""))
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
    ))
}
