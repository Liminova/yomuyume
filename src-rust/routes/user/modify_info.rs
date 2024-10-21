use std::sync::Arc;

use crate::{models::prelude::*, routes::check_pass, AppError, AppState};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ModifyRequestBody {
    pub username: Option<String>,
    pub email: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

/// Modify user information.
#[utoipa::path(post, path = "/api/user/modify", responses(
    (status = 200, description = "Modify user successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String)
))]
pub async fn post_modify_info(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(body): Json<ModifyRequestBody>,
) -> Result<Response, AppError> {

    let current_password_hash = user.password_hash.clone();
    let mut active_user: users::ActiveModel = user.into();

    if let Some(username) = body.username {
        active_user.username = Set(username);
    }

    if let Some(email) = body.email {
        active_user.email = Set(email);
        active_user.verified_at = Set(None);
    }

    match (body.current_password, body.new_password) {
        (None, Some(_)) => {
            return Ok((
                StatusCode::BAD_REQUEST,
                "current password is required to change password",
            )
                .into_response());
        }
        (Some(current_password), Some(new_password)) => {
            if !check_pass(&current_password_hash, &current_password) {
                return Ok((StatusCode::BAD_REQUEST, "invalid current password").into_response());
            }
            active_user.password_hash = Set(new_password);
        }
        (_, _) => {}
    }

    if active_user.is_changed() {
        active_user.updated_at = Set(Some(chrono::Utc::now()));
        active_user
            .save(&app_state.db)
            .await
            .map_err(|e| AppError::from(anyhow::anyhow!("can't modify user: {}", e)))?;
    }

    Ok((StatusCode::OK).into_response())
}
