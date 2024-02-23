use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension,
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::warn;

use crate::{
    models::prelude::*,
    routes::{ErrRsp, GenericRsp},
    AppState,
};

#[utoipa::path(put, path = "/api/user/progress/:title_id/:page", responses(
    (status = 200, description = "Set progress successfully", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
))]
pub async fn put_progress(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path((title_id, page)): Path<(String, i64)>,
) -> Result<impl IntoResponse, ErrRsp> {
    let progress_model = Progresses::find()
        .filter(progresses::Column::TitleId.eq(&title_id))
        .filter(progresses::Column::UserId.eq(&user.id))
        .one(&data.db)
        .await
        .map_err(|e| {
            warn!(
                "find progress failed | title {} | user {}: {}",
                title_id, user.id, e
            );
            ErrRsp::internal(format!("Can't find progress: {}", e))
        })?;

    // Update if exist
    if let Some(progress_model) = progress_model {
        let mut active_model: progresses::ActiveModel = progress_model.into();
        active_model.last_read_at = Set(chrono::Utc::now().to_rfc3339());
        active_model.page = Set(page);
        active_model.update(&data.db).await.map_err(|e| {
            warn!(
                "update progress failed | title {} | user {}: {}",
                title_id, user.id, e
            );
            ErrRsp::internal(format!("Can't update progress: {}", e))
        })?;

        return Ok(GenericRsp::create("Progress updated."));
    }

    progresses::ActiveModel {
        id: NotSet,
        user_id: Set(user.id.clone()),
        title_id: Set(title_id.clone()),
        last_read_at: Set(chrono::Utc::now().to_rfc3339()),
        page: Set(page),
    }
    .insert(&data.db)
    .await
    .map_err(|e| {
        warn!(
            "insert progress failed | title {} | user {}: {}",
            title_id, user.id, e
        );
        ErrRsp::internal(format!("Can't insert progress: {}", e))
    })?;

    // just return the OK status
    Ok(GenericRsp::create("Progress set."))
}
