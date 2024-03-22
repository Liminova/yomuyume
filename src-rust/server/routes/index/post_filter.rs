use std::{path::PathBuf, sync::Arc};

pub use bridge::routes::index::{FilterRequest, FilterResponseBody, FilterTitleResponseBody};

use super::{find_favorite_count, find_page_count, find_page_read};
use crate::{
    models::prelude::*,
    routes::{calculate_dimension, MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{extract::State, http::HeaderMap, response::IntoResponse, Extension, Json};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, Order, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};

/// Filtering titles by various parameters.
///
/// And also sorting them by various options.
#[utoipa::path(post, path = "api/index/filter", responses(
    (status = 200, description = "Fetch all items successful", body = FilterResponseBody),
    (status = 204, description = "Fetch all items successful, but none were found", body = FilterResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn post_filter(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    header: HeaderMap,
    Json(query): Json<FilterRequest>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let keywords = query.keywords;
    let category_ids = query.category_ids;
    let tag_ids = query.tag_ids;
    let limit = query.limit;

    if keywords.is_none() && category_ids.is_none() && tag_ids.is_none() {
        return Ok(builder.no_content("No title found"));
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
            .map_err(|e| builder.db_error(e))?;
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
                .map_err(|e| builder.db_error(e))?;
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
                .map_err(|e| builder.db_error(e))?;
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
                .map_err(|e| builder.db_error(e))?;
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
                .map_err(|e| builder.db_error(e))?;
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
        .map_err(|e| builder.db_error(e))?;

    let mut resp_data: Vec<FilterTitleResponseBody> = vec![];

    for title in title_models {
        let page_count = find_page_count(&data.db, &title.id).await;
        let favorite_count = find_favorite_count(&data.db, &title.id).await;
        let page_read = find_page_read(&data.db, &title.id, &user.id).await;
        let cover_model = Covers::find_by_id(&title.id)
            .one(&data.db)
            .await
            .map_err(|e| builder.db_error(e))?
            .ok_or_else(|| builder.no_content("No cover found"))?;

        let (width, height) = calculate_dimension(cover_model.ratio);

        resp_data.push(FilterTitleResponseBody {
            id: title.id.to_string(),
            title: title.title,
            author: title.author,
            category_id: title.category_id,
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

    Ok(builder.success(FilterResponseBody { data: resp_data }))
}
