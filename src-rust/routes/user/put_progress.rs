use std::sync::Arc;

use crate::{models::prelude::*, AppError, AppState};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
};

#[utoipa::path(put, path = "/api/user/progress/{title_id}/{page}", responses(
    (status = 200, description = "Set progress successfully"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn put_progress(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Path((title_id, page)): Path<(String, i64)>,
) -> Result<Response, AppError> {
    let title_id = match CustomID::from(title_id) {
        Ok(id) => id,
        Err(e) => return Ok((StatusCode::BAD_REQUEST, e).into_response()),
    };

    let progress_model = Progresses::find()
        .filter(
            Condition::all()
                .add(progresses::Column::TitleId.eq(&title_id))
                .add(progresses::Column::UserId.eq(&user.id)),
        )
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find progress: {}", e)))?;

    if let Some(progress_model) = progress_model {
        let mut active_model: progresses::ActiveModel = progress_model.into();
        active_model.last_read_at = Set(Some(chrono::Utc::now()));
        active_model.page = Set(page);
        active_model
            .update(&app_state.db)
            .await
            .map_err(|e| AppError::from(anyhow::anyhow!("can't update progress: {}", e)))?;
    } else {
        progresses::ActiveModel {
            id: NotSet,
            user_id: Set(user.id),
            title_id: Set(title_id),
            last_read_at: Set(Some(chrono::Utc::now())),
            page: Set(page),
        }
        .insert(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't insert progress: {}", e)))?;
    }

    Ok((StatusCode::OK).into_response())
}
