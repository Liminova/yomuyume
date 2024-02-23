use crate::{
    models::prelude::*,
    routes::{ErrRsp, GenericRsp},
    AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
};
use std::sync::Arc;

#[utoipa::path(put, path = "/api/user/favorite/:id", responses(
    (status = 200, description = "Add favorite successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn put_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ErrRsp> {
    let title = Titles::find_by_id(id)
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't find title: {}", e)))?
        .ok_or_else(|| ErrRsp::bad_request("Invalid title id."))?;

    let favorite_model = Favorites::find()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.eq(&title.id))
                .add(favorites::Column::UserId.eq(&user.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't find favorite: {}", e)))?;

    if favorite_model.is_some() {
        return Err(ErrRsp::bad_request("Title already favorited."));
    }

    let _ = favorites::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    }
    .insert(&data.db)
    .await
    .map_err(|e| ErrRsp::internal(format!("Can't insert favorite: {}", e)))?;

    Ok(GenericRsp::create("Add favorite successful."))
}

#[utoipa::path(put, path = "/api/user/bookmark/:id", responses(
    (status = 200, description = "Add bookmark successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn put_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ErrRsp> {
    let title = Titles::find_by_id(id)
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't find title: {}", e)))?
        .ok_or_else(|| ErrRsp::bad_request("Invalid title id."))?;

    let bookmark_model = Bookmarks::find()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.eq(&title.id))
                .add(bookmarks::Column::UserId.eq(&user.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't find bookmark: {}", e)))?;

    if bookmark_model.is_some() {
        return Err(ErrRsp::bad_request("Title already bookmarked."));
    }

    let _ = bookmarks::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    }
    .insert(&data.db)
    .await
    .map_err(|e| ErrRsp::internal(format!("Can't insert bookmark: {}", e)))?;

    Ok(GenericRsp::create("Add bookmark successful."))
}

#[utoipa::path(delete, path = "/api/user/favorite/:id", responses(
    (status = 200, description = "Delete favorite successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn delete_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ErrRsp> {
    let title = Titles::find_by_id(id)
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't find title: {}", e)))?
        .ok_or_else(|| ErrRsp::bad_request("Invalid title id."))?;

    Favorites::delete_many()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.contains(&title.id))
                .add(favorites::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't delete favorite: {}", e)))?;

    Ok(GenericRsp::create("Delete favorite successful."))
}

#[utoipa::path(delete, path = "/api/user/bookmark/:id", responses(
    (status = 200, description = "Delete bookmark successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn delete_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ErrRsp> {
    let title = Titles::find_by_id(id)
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't find title: {}", e)))?
        .ok_or_else(|| ErrRsp::bad_request("Invalid title id."))?;

    Bookmarks::delete_many()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.contains(&title.id))
                .add(bookmarks::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't delete bookmark: {}", e)))?;

    Ok(GenericRsp::create("Delete bookmark successful."))
}
