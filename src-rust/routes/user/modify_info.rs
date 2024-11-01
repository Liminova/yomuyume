use std::sync::Arc;

use crate::{
    routes::{check_pass, hash_pass},
    AppError, AppState,
};

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
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
    Extension(user_id): Extension<String>,
    Json(body): Json<ModifyRequestBody>,
) -> Result<Response, AppError> {
    let mut new_password_hash = String::new();
    match (body.current_password, body.new_password) {
        (None, Some(_)) => {
            return Ok((
                StatusCode::BAD_REQUEST,
                "current password is required to change password",
            )
                .into_response());
        }
        (Some(current_password), Some(new_password)) => {
            let current_password_hash = sqlx::query!(
                r#"SELECT password_hash FROM users WHERE id = $1"#,
                user_id.as_str()
            )
            .fetch_one(&app_state.pool)
            .await
            .context("can't get user")?
            .password_hash;
            if !check_pass(&current_password_hash, &current_password) {
                return Ok((StatusCode::BAD_REQUEST, "invalid current password").into_response());
            }
            new_password_hash = hash_pass(new_password)?;
        }
        (_, _) => {}
    }

    sqlx::query!(
        r#"UPDATE users SET
            username = CASE WHEN $1 = '' THEN username ELSE $1 END,
            email = CASE WHEN $2 = '' THEN email ELSE $2 END,
            password_hash = CASE WHEN $3 = '' THEN password_hash ELSE $3 END,
            updated_at = NOW() WHERE id = $4"#,
        body.username.unwrap_or_default(),
        body.email.unwrap_or_default(),
        new_password_hash,
        user_id.as_str()
    )
    .execute(&app_state.pool)
    .await
    .context("can't update user")?;

    Ok((StatusCode::OK).into_response())
}
