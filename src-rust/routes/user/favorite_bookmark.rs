use std::sync::Arc;

use crate::{models::prelude::*, AppError, AppState, GenericResponseBody};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
};

#[utoipa::path(put, path = "/api/user/favorite/{id}", responses(
    (status = 200, description = "Add favorite successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn put_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let title_id = match CustomID::from(id) {
        Ok(id) => id,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e).into_response()),
    };

    let title = match Titles::find_by_id(title_id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find title: {}", e)))?
    {
        Some(title) => title,
        None => return Ok((StatusCode::BAD_REQUEST, "Invalid title id.").into_response()),
    };

    let favorite_model = Favorites::find()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.eq(&title.id))
                .add(favorites::Column::UserId.eq(&user.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find favorite: {}", e)))?;

    if favorite_model.is_some() {
        return Ok((StatusCode::BAD_REQUEST, "Title already favorited.").into_response());
    }

    let _ = favorites::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    }
    .insert(&data.db)
    .await
    .map_err(|e| AppError::from(anyhow::anyhow!("Can't insert favorite: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("Add favorite successful.")),
    )
        .into_response())
}

#[utoipa::path(put, path = "/user/favorite/{id}", responses(
    (status = 200, description = "Add bookmark successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn put_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let title_id = match CustomID::from(id) {
        Ok(id) => id,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e).into_response()),
    };

    let title = match Titles::find_by_id(title_id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find title: {}", e)))?
    {
        Some(title) => title,
        None => return Ok((StatusCode::BAD_REQUEST, "Invalid title id.").into_response()),
    };

    let bookmark_model = Bookmarks::find()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.eq(&title.id))
                .add(bookmarks::Column::UserId.eq(&user.id)),
        )
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find bookmark: {}", e)))?;

    if bookmark_model.is_some() {
        return Ok((StatusCode::BAD_REQUEST, "Title already bookmarked.").into_response());
    }

    let _ = bookmarks::ActiveModel {
        id: NotSet,
        title_id: Set(title.id),
        user_id: Set(user.id),
    }
    .insert(&data.db)
    .await
    .map_err(|e| AppError::from(anyhow::anyhow!("Can't insert bookmark: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("Add bookmark successful.")),
    )
        .into_response())
}

#[utoipa::path(delete, path = "/api/user/favorite/{id}", responses(
    (status = 200, description = "Delete favorite successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn delete_favorite(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let title_id = match CustomID::from(id) {
        Ok(id) => id,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e).into_response()),
    };

    let title = match Titles::find_by_id(&title_id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find title: {}", e)))?
    {
        Some(title) => title,
        None => return Ok((StatusCode::BAD_REQUEST, "Invalid title id.").into_response()),
    };

    Favorites::delete_many()
        .filter(
            Condition::all()
                .add(favorites::Column::TitleId.contains(&title.id))
                .add(favorites::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't delete favorite: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("Delete favorite successful.")),
    )
        .into_response())
}

#[utoipa::path(delete, path = "/user/favorite/{id}", responses(
    (status = 200, description = "Delete bookmark successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn delete_bookmark(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let id = match CustomID::from(id) {
        Ok(id) => id,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e).into_response()),
    };

    let title = match Titles::find_by_id(id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find title: {}", e)))?
    {
        Some(title) => title,
        None => return Ok((StatusCode::BAD_REQUEST, "Invalid title id.").into_response()),
    };

    Bookmarks::delete_many()
        .filter(
            Condition::all()
                .add(bookmarks::Column::TitleId.contains(&title.id))
                .add(bookmarks::Column::UserId.contains(&user.id)),
        )
        .exec(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't delete bookmark: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("Delete bookmark successful.")),
    )
        .into_response())
}
