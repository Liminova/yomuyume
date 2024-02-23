use super::{find_favorite_count, find_page_count, find_page_read};
use crate::{
    models::prelude::*,
    routes::{calculate_dimension, ErrRsp},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{path::PathBuf, sync::Arc};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct FilterRequest {
    /// Keywords to search for (search in title, description, author, tags)
    keywords: Option<Vec<String>>,
    /// Categories to filter by
    category_ids: Option<Vec<String>>,
    /// Tags to filter by
    tag_ids: Option<Vec<i32>>,
    /// Maximum number of results to return
    limit: Option<u32>,

    is_reading: Option<bool>,
    is_finished: Option<bool>,
    is_bookmarked: Option<bool>,
    is_favorite: Option<bool>,

    sort_by: Option<String>,
    sort_order: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[skip_serializing_none]
pub struct FilterTitleResponseBody {
    id: String,
    title: String,
    author: Option<String>,
    category_id: String,
    release: Option<String>,
    favorite_count: Option<i64>,
    page_count: i64,
    page_read: Option<i64>,

    /// Thumbnail
    blurhash: String,
    width: u32,
    height: u32,
    format: String,
}

#[derive(Serialize, ToSchema)]
pub struct FilterResponseBody {
    pub data: Vec<FilterTitleResponseBody>,
}

/// Filtering titles by various parameters.
///
/// And also sorting them by various options.
#[utoipa::path(post, path = "/api/index/filter", responses(
    (status = 200, description = "Fetch all items successful", body = FilterResponseBody),
    (status = 204, description = "Fetch all items successful, but none were found", body = FilterResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn post_filter(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<FilterRequest>,
) -> Result<impl IntoResponse, ErrRsp> {
    let keywords = query.keywords;
    let category_ids = query.category_ids;
    let tag_ids = query.tag_ids;
    let limit = query.limit;

    if keywords.is_none() && category_ids.is_none() && tag_ids.is_none() {
        return Ok((
            StatusCode::NO_CONTENT,
            Json(FilterResponseBody { data: vec![] }),
        ));
    }

    let mut condition = Condition::any();

    if let Some(category_ids) = category_ids {
        for category_id in category_ids {
            condition = condition.add(titles::Column::CategoryId.eq(category_id));
        }
    }

    if let Some(keywords) = keywords {
        for keyword in keywords {
            condition = condition
                .add(titles::Column::Title.contains(keyword.to_lowercase()))
                .add(titles::Column::Author.contains(keyword.to_lowercase()))
                .add(titles::Column::Description.contains(keyword.to_lowercase()));
        }
    }

    if let Some(tag_ids) = tag_ids {
        let mut internal_cond = Condition::any();
        for tag_id in tag_ids {
            internal_cond = internal_cond.add(titles_tags::Column::TagId.eq(tag_id));
        }
        let title_tag_has_tag_id = TitlesTags::find()
            .filter(internal_cond)
            .all(&data.db)
            .await
            .map_err(ErrRsp::db)?;
        for entity in title_tag_has_tag_id {
            condition = condition.add(titles::Column::Id.eq(entity.title_id));
        }
    }

    if let Some(is_reading) = query.is_reading {
        if is_reading {
            let progress_models = Progresses::find()
                .filter(progresses::Column::UserId.eq(&user.id))
                .filter(progresses::Column::Page.gt(0))
                .all(&data.db)
                .await
                .map_err(ErrRsp::db)?;
            for entity in progress_models {
                condition = condition.add(titles::Column::Id.eq(entity.title_id));
            }
        }
    }

    if let Some(is_finished) = query.is_finished {
        if is_finished {
            let progress_models = Progresses::find()
                .filter(progresses::Column::UserId.eq(&user.id))
                .filter(progresses::Column::Page.eq(0))
                .all(&data.db)
                .await
                .map_err(ErrRsp::db)?;
            for entity in progress_models {
                condition = condition.add(titles::Column::Id.eq(entity.title_id));
            }
        }
    }

    if let Some(is_bookmarked) = query.is_bookmarked {
        if is_bookmarked {
            let bookmark_models = Bookmarks::find()
                .filter(bookmarks::Column::UserId.eq(&user.id))
                .all(&data.db)
                .await
                .map_err(ErrRsp::db)?;
            for entity in bookmark_models {
                condition = condition.add(titles::Column::Id.eq(entity.title_id));
            }
        }
    }

    if let Some(is_favorite) = query.is_favorite {
        if is_favorite {
            let favorite_models = Favorites::find()
                .filter(favorites::Column::UserId.eq(&user.id))
                .all(&data.db)
                .await
                .map_err(ErrRsp::db)?;
            for entity in favorite_models {
                condition = condition.add(titles::Column::Id.eq(entity.title_id));
            }
        }
    }

    let sort_by = match &query.sort_by {
        Some(sort_by) => match sort_by.as_str() {
            "alphabetical" => titles::Column::Title,
            "add date" => titles::Column::DateAdded,
            "release date" => titles::Column::Release,
            "update date" => titles::Column::DateUpdated,
            // "last read" => {},
            _ => titles::Column::Title,
        },
        None => titles::Column::Title,
    };

    let sort_order = match &query.sort_order {
        Some(sort_order) => match sort_order.as_str() {
            "ascending" => Order::Asc,
            "descending" => Order::Desc,
            _ => Order::Asc,
        },
        None => Order::Asc,
    };

    let title_models = Titles::find()
        .apply_if(limit.map(|limit| limit as u64), QuerySelect::limit)
        .filter(condition)
        .order_by(sort_by, sort_order)
        .all(&data.db)
        .await
        .map_err(ErrRsp::db)?;

    let mut resp_data: Vec<FilterTitleResponseBody> = vec![];

    for title in title_models {
        let page_count = find_page_count(&data.db, &title.id).await;
        let favorite_count = find_favorite_count(&data.db, &title.id).await;
        let page_read = find_page_read(&data.db, &title.id, &user.id).await;
        let thumbnail_model = Thumbnails::find_by_id(&title.id)
            .one(&data.db)
            .await
            .map_err(ErrRsp::db)?
            .ok_or_else(|| {
                ErrRsp::new(StatusCode::INTERNAL_SERVER_ERROR, "Thumbnail not found.")
            })?;

        let (width, height) = calculate_dimension(thumbnail_model.ratio);

        resp_data.push(FilterTitleResponseBody {
            id: title.id,
            title: title.title,
            author: title.author,
            category_id: title.category_id,
            release: title.release,
            favorite_count,
            page_count,
            page_read,

            blurhash: thumbnail_model.blurhash,
            width,
            height,
            format: PathBuf::from(thumbnail_model.path)
                .extension()
                .map(|s| s.to_str().unwrap_or(""))
                .unwrap_or("")
                .to_ascii_lowercase(),
        });
    }

    let status_code = match resp_data.is_empty() {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::OK,
    };

    Ok((status_code, Json(FilterResponseBody { data: resp_data })))
}
