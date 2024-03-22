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
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};

#[utoipa::path(put, path = "/api/user/progress/{title_id}/{page}", responses(
    (status = 200, description = "Set progress successfully", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
))]
pub async fn put_progress(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    header: HeaderMap,
    Path((title_id, page)): Path<(String, i64)>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let title_id = TitleID::from(title_id).map_err(|e| builder.bad_id(e))?;

    let progress_model = Progresses::find()
        .filter(progresses::Column::TitleId.eq(&title_id))
        .filter(progresses::Column::UserId.eq(&user.id))
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    // Update if exist
    if let Some(progress_model) = progress_model {
        let mut active_model: progresses::ActiveModel = progress_model.into();
        active_model.last_read_at = Set(chrono::Utc::now().to_rfc3339());
        active_model.page = Set(page);
        active_model
            .update(&data.db)
            .await
            .map_err(|e| builder.db_error(e))?;

        return Ok(builder.generic_success("Progress updated."));
    }

    progresses::ActiveModel {
        id: NotSet,
        user_id: Set(user.id),
        title_id: Set(title_id),
        last_read_at: Set(chrono::Utc::now().to_rfc3339()),
        page: Set(page),
    }
    .insert(&data.db)
    .await
    .map_err(|e| builder.db_error(e))?;

    // just return the OK status
    Ok(builder.generic_success("Progress set."))
}
