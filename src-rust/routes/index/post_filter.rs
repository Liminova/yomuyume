use std::{path::PathBuf, sync::Arc};

use super::{find_favorite_count, find_page_count, find_page_read};
use crate::{models::prelude::*, routes::calculate_dimension, AppError, AppState};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FilterRequestBody {
    /// Keywords to search for (search in title, description, author, tags)
    pub keywords: Option<Vec<String>>,
    /// Categories to filter by
    pub category_ids: Option<Vec<String>>,
    /// Tags to filter by
    pub tag_ids: Option<Vec<i32>>,
    /// Maximum number of results to return
    pub limit: Option<u32>,

    pub is_reading: Option<bool>,
    pub is_finished: Option<bool>,
    pub is_bookmarked: Option<bool>,
    pub is_favorite: Option<bool>,

    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde_with::skip_serializing_none]
pub struct FilterTitleResponseBody {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub category_id: Option<String>,
    pub release: Option<String>,
    pub favorite_count: Option<i64>,
    pub page_count: i64,
    pub page_read: Option<i64>,

    /// Cover
    pub blurhash: Option<String>,
    pub blurhash_width: Option<u8>,
    pub blurhash_height: Option<u8>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FilterResponseBody {
    pub data: Vec<FilterTitleResponseBody>,
}

/// Filtering titles by various parameters.
///
/// And also sorting them by various options.
#[utoipa::path(post, path = "api/index/filter", responses(
    (status = 200, description = "Fetch all items successful", body = FilterResponseBody),
    (status = 204, description = "Fetch all items successful, but none were found", body = FilterResponseBody),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn post_filter(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<FilterRequestBody>,
) -> Result<Response, AppError> {
    let keywords = query.keywords;
    let category_ids = query.category_ids;
    let tag_ids = query.tag_ids;
    let limit = query.limit;

    if keywords.is_none() && category_ids.is_none() && tag_ids.is_none() {
        return Ok((StatusCode::NO_CONTENT, "no title found").into_response());
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
            .all(&app_state.db)
            .await
            .map_err(|e| AppError::from(anyhow::anyhow!("can't find tags: {}", e)))?;
        for entity in title_tag_has_tag_id {
            condition = condition.add(titles::Column::Id.eq(entity.title_id));
        }
    }

    if let Some(is_reading) = query.is_reading {
        if is_reading {
            let progress_models = Progresses::find()
                .filter(progresses::Column::UserId.eq(&user.id))
                .filter(progresses::Column::Page.gt(0))
                .all(&app_state.db)
                .await
                .map_err(|e| AppError::from(anyhow::anyhow!("can't find progress: {}", e)))?;
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
                .all(&app_state.db)
                .await
                .map_err(|e| AppError::from(anyhow::anyhow!("can't find progress: {}", e)))?;
            for entity in progress_models {
                condition = condition.add(titles::Column::Id.eq(entity.title_id));
            }
        }
    }

    if let Some(is_bookmarked) = query.is_bookmarked {
        if is_bookmarked {
            let bookmark_models = Bookmarks::find()
                .filter(bookmarks::Column::UserId.eq(&user.id))
                .all(&app_state.db)
                .await
                .map_err(|e| AppError::from(anyhow::anyhow!("can't find bookmark: {}", e)))?;
            for entity in bookmark_models {
                condition = condition.add(titles::Column::Id.eq(entity.title_id));
            }
        }
    }

    if let Some(is_favorite) = query.is_favorite {
        if is_favorite {
            let favorite_models = Favorites::find()
                .filter(favorites::Column::UserId.eq(&user.id))
                .all(&app_state.db)
                .await
                .map_err(|e| AppError::from(anyhow::anyhow!("can't find favorite: {}", e)))?;
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
        .all(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find titles: {}", e)))?;

    let mut resp_data: Vec<FilterTitleResponseBody> = vec![];

    for title in title_models {
        let page_count = find_page_count(&app_state.db, &title.id).await;
        let favorite_count = find_favorite_count(&app_state.db, &title.id).await;
        let page_read = find_page_read(&app_state.db, &title.id, &user.id).await;
        let cover_model = match Covers::find_by_id(&title.id)
            .one(&app_state.db)
            .await
            .map_err(|e| AppError::from(anyhow::anyhow!("Can't find cover: {}", e)))?
        {
            Some(cover) => cover,
            None => return Ok((StatusCode::NO_CONTENT, "No cover found").into_response()),
        };

        let (width, height) = calculate_dimension(&app_state.config, cover_model.ratio);

        resp_data.push(FilterTitleResponseBody {
            id: title.id.to_string(),
            title: title.title,
            author: title.author,
            category_id: title.category_id.to_string(),
            release: title.release,
            favorite_count,
            page_count,
            page_read,

            blurhash: cover_model.blurhash,
            width,
            height,
            format: PathBuf::from(cover_model.path)
                .extension()
                .map(|s| s.to_str().unwrap_or(""))
                .unwrap_or("")
                .to_ascii_lowercase(),
        });
    }

    Ok((StatusCode::OK, Json(FilterResponseBody { data: resp_data })).into_response())
}
