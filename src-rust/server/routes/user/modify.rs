use std::sync::Arc;

pub use bridge::routes::user::ModifyRequest;

use crate::{
    models::prelude::*,
    routes::{check_pass, MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{extract::State, http::HeaderMap, response::IntoResponse, Extension, Json};
use sea_orm::{ActiveModelTrait, Set};

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
    header: HeaderMap,
    Json(body): Json<ModifyRequest>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

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
            return Err(builder.unauthorized("User is not verified, cannot change password."));
        }
        if !check_pass(&password_in_db, &password) {
            return Err(builder.bad_request("Invalid password."));
        }
    }

    if let Some(new_password) = body.new_password {
        active_user.password = Set(new_password);
    }

    active_user
        .save(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    Ok(builder.generic_success("Modify user successful."))
}
