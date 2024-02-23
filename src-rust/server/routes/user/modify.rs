use crate::{
    models::prelude::*,
    routes::{check_pass, ErrRsp, GenericRsp},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct ModifyRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub new_password: Option<String>,
}

/// Modify user information.
#[utoipa::path(post, path = "/api/user/modify", responses(
    (status = 200, description = "Modify user successful", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody)
))]
pub async fn post_modify(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(body): Json<ModifyRequest>,
) -> Result<impl IntoResponse, ErrRsp> {
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
            return Err(ErrRsp::new(
                StatusCode::UNAUTHORIZED,
                "User is not verified, cannot change password.",
            ));
        }
        if !check_pass(&password_in_db, &password) {
            return Err(ErrRsp::bad_request("Invalid password."));
        }
    }

    if let Some(new_password) = body.new_password {
        active_user.password = Set(new_password);
    }

    active_user
        .save(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't update user: {}", e)))?;

    Ok(GenericRsp::create("Modify user successful."))
}
