use std::sync::Arc;

use crate::{models::prelude::*, routes::check_pass, AppError, AppState, GenericResponseBody};

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
pub struct ModifyRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub new_password: Option<String>,
}

/// Modify user information.
#[utoipa::path(post, path = "/api/user/modify", responses(
    (status = 200, description = "Modify user successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn post_modify(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(body): Json<ModifyRequest>,
) -> Result<Response, AppError> {
    let password_in_db = user.password.clone();
    let is_verified = user.is_verified;

    let mut active_user: users::ActiveModel = user.into();

    if let Some(username) = body.username {
        active_user.username = Set(username);
    }

    if let Some(email) = body.email {
        active_user.email = Set(email);
        active_user.is_verified = Set(false);
    }

    if let Some(password) = body.password {
        if !is_verified {
            return Ok((
                StatusCode::UNAUTHORIZED,
                "User is not verified, cannot change password.",
            )
                .into_response());
        }
        if !check_pass(&password_in_db, &password) {
            return Ok((StatusCode::BAD_REQUEST, "Invalid password.").into_response());
        }
    }

    if let Some(new_password) = body.new_password {
        active_user.password = Set(new_password);
    }

    active_user
        .save(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't modify user: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("Modify user successful.")),
    )
        .into_response())
}
