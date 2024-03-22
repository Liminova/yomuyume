use std::{path::PathBuf, sync::Arc};

pub use bridge::routes::index::{ResponseCover, ResponsePage, TitleResponseBody};

use crate::{
    models::prelude::*,
    routes::{calculate_dimension, MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension,
};
use sea_orm::*;

/// Get everything about a title.
#[utoipa::path(get, path = "/api/index/title/{title_id}", responses(
    (status = 200, description = "Fetch title successful", body = TitleResponseBody),
    (status = 204, description = "No title found for the given id", body = TitleResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn get_title(
    State(data): State<Arc<AppState>>,
    Path(title_id): Path<String>,
    header: HeaderMap,
    Extension(user): Extension<users::Model>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let title_id =
        CustomID::from(title_id).map_err(|_| builder.bad_request("Invalid title id."))?;

    let title = Titles::find_by_id(title_id)
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.not_found("No title found."))?;

    let cover = Covers::find_by_id(&title.id)
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.not_found("No cover found."))?;

    let pages = Pages::find()
        .filter(pages::Column::TitleId.eq(&title.id))
        .order_by_asc(pages::Column::Path)
        .all(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    // place the cover.path at the front of the Vec<pages::Model>
    // and convert it to Vec<ResponsePage>
    let pages = pages
        .into_iter()
        .fold(Vec::new(), |mut list, page_model| {
            if page_model.path == cover.path {
                list.insert(0, page_model);
            } else {
                list.push(page_model);
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
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .map(|_| true);

    let is_bookmark = Bookmarks::find()
        .filter(
            Condition::all()
                .add(bookmarks::Column::UserId.eq(&user.id))
                .add(bookmarks::Column::TitleId.eq(&title.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .map(|_| true);

    let page_read = Progresses::find()
        .filter(
            Condition::all()
                .add(progresses::Column::UserId.eq(&user.id))
                .add(progresses::Column::TitleId.eq(&title.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .map(|p| p.page);

    let favorites = match Favorites::find()
        .filter(favorites::Column::TitleId.eq(&title.id))
        .count(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
    {
        0 => None,
        n => Some(n as i64),
    };

    let bookmarks = match Bookmarks::find()
        .filter(bookmarks::Column::TitleId.eq(&title.id))
        .count(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
    {
        0 => None,
        n => Some(n as i64),
    };

    let tag_ids = TitlesTags::find()
        .filter(titles_tags::Column::TitleId.eq(&title.id))
        .all(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .iter()
        .map(|tag| tag.tag_id)
        .collect::<Vec<_>>();

    let (width, height) = calculate_dimension(cover.ratio);

    let data = TitleResponseBody {
        category_id: title.category_id,
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
    };

    Ok(builder.success(data))
}
