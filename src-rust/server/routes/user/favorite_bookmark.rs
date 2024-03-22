use std::sync::Arc;

use crate::{
    models::prelude::*,
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
};

use self::titles::TitleID;

#[utoipa::path(put, path = "/api/user/favorite/{id}", responses(
    (status = 200, description = "Add favorite successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn put_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    header: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let title_id = TitleID::from(id).map_err(|e| builder.bad_id(e))?;

    let title = Titles::find_by_id(title_id)
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.bad_request("Invalid title id."))?;

    let favorite_model = Favorites::find()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.eq(&title.id))
                .add(favorites::Column::UserId.eq(&user.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    if favorite_model.is_some() {
        return Ok(builder.bad_request("Title already favorited."));
    }

    let _ = favorites::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    }
    .insert(&data.db)
    .await
    .map_err(|e| builder.db_error(e))?;

    Ok(builder.generic_success("Add favorite successful."))
}

#[utoipa::path(put, path = "/user/favorite/{id}", responses(
    (status = 200, description = "Add bookmark successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn put_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    header: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let title_id = TitleID::from(id).map_err(|e| builder.bad_id(e))?;

    let title = Titles::find_by_id(title_id)
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.bad_request("Invalid title id."))?;

    let bookmark_model = Bookmarks::find()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.eq(&title.id))
                .add(bookmarks::Column::UserId.eq(&user.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    if bookmark_model.is_some() {
        return Ok(builder.generic_success("Title already bookmarked."));
    }

    let _ = bookmarks::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    }
    .insert(&data.db)
    .await
    .map_err(|e| builder.db_error(e))?;

    Ok(builder.generic_success("Add bookmark successful."))
}

#[utoipa::path(delete, path = "/api/user/favorite/{id}", responses(
    (status = 200, description = "Delete favorite successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn delete_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    header: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let title_id = TitleID::from(id).map_err(|e| builder.bad_id(e))?;

    let title = Titles::find_by_id(&title_id)
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.bad_id(title_id))?;

    Favorites::delete_many()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.contains(&title.id))
                .add(favorites::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    Ok(builder.generic_success("Delete favorite successful."))
}

#[utoipa::path(delete, path = "/user/favorite/{id}", responses(
    (status = 200, description = "Delete bookmark successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn delete_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    header: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let id = CustomID::from(id).map_err(|e| builder.bad_request(e))?;

    let title = Titles::find_by_id(id)
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.bad_request("Invalid title id."))?;

    Bookmarks::delete_many()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.contains(&title.id))
                .add(bookmarks::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    Ok(builder.generic_success("Delete bookmark successful."))
}
