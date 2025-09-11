use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    AppState,
    routes::{
        UserIDExtension, check_pass,
        errors::{InternalError, RequestError},
        hash_pass,
    },
    utils::constants::POST_USER_MODIFY_PATH,
};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ModifyRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub old_password: Option<String>,
    pub new_password: Option<String>,
}

/// Modify user information
#[utoipa::path(
    post,
    path = POST_USER_MODIFY_PATH,
    responses(
        (status = 200, description = "Modify user successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(("session-secret" = []))
)]
pub async fn post_modify(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
    Json(body): Json<ModifyRequest>,
) -> Result<Response, InternalError> {
    let Some(user_id) = user_id.0 else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    let username = body.username.unwrap_or_default();
    let email = body.email.unwrap_or_default();

    sqlx::query!(
        "UPDATE users SET
            username = CASE
                WHEN :username = '' THEN username
                ELSE :username
            END,
            email = CASE
                WHEN :email = '' THEN email
                ELSE :email
            END
        WHERE id = :id",
        username,
        email,
        user_id
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't upsert user info: {e}"))?;

    if let Some(old_password) = body.old_password {
        let Some(new_password) = body.new_password else {
            return Ok((
                StatusCode::BAD_REQUEST,
                RequestError::CurrentPasswordRequired,
            )
                .into_response());
        };

        let password_hash = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", user_id)
            .fetch_one(&app_state.pool)
            .await
            .inspect_err(|e| error!("can't query user by id: {e}"))
            .map(|r| r.password_hash)?;

        if !check_pass(password_hash, old_password) {
            return Ok((
                StatusCode::BAD_REQUEST,
                RequestError::InvalidCurrentPassword,
            )
                .into_response());
        }

        let new_password_hash = hash_pass(new_password).map_err(|e| {
            error!("can't hash new password: {e}");
            InternalError::PasswordHash(e)
        })?;

        sqlx::query!(
            "UPDATE users SET password_hash = ? WHERE id = ?",
            new_password_hash,
            user_id
        )
        .execute(&app_state.pool)
        .await
        .inspect_err(|e| error!("can't update user password: {e}"))?;
    }

    Ok(StatusCode::OK.into_response())
}
