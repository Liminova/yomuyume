use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::{check_pass, hash_pass},
    types::UserID,
    utils::{app_error::AppError, app_state::AppState},
};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ModifyRequestBody {
    pub username: Option<String>,
    pub email: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

/// modify user info
#[utoipa::path(post, path = "/api/user/modify", responses(
    (status = 200, description = "modify user successful"),
    (status = 400, description = "bad request", body = String),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String)
), security(("session-id" = [], "session-secret" = [])))]
pub async fn post_modify(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
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
                "SELECT password_hash
                FROM users
                WHERE id = $1",
                user_id.as_ref()
            )
            .fetch_one(&app_state.pool)
            .await
            .map_err(|e| {
                tracing::error!("{e:?}");
                AppError::DB(e)
            })?
            .password_hash;
            if !check_pass(&current_password_hash, &current_password) {
                return Ok((StatusCode::BAD_REQUEST, "invalid current password").into_response());
            }
            new_password_hash = hash_pass(new_password)?;
        }
        _ => {}
    }

    let username = body.username.unwrap_or_default();
    let email = body.email.unwrap_or_default();
    let now = Utc::now();

    sqlx::query!(
        "UPDATE users
        SET username = CASE
                WHEN $1 = '' THEN username
                ELSE $1
            END,
            email = CASE
                WHEN $2 = '' THEN email
                ELSE $2
            END,
            password_hash = CASE
                WHEN $3 = '' THEN password_hash
                ELSE $3
            END,
            updated_at = $4
        WHERE id = $5",
        &username,
        &email,
        new_password_hash,
        &now,
        user_id.as_ref()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        AppError::DB(e)
    })?;

    let mut user = app_state.user_cache.get_mut(&user_id).ok_or_else(|| {
        let e = AppError::WriteCache("user".to_string());
        tracing::error!("{e:?}");
        e
    })?;
    user.value_mut().username = username;
    user.value_mut().email = email;
    user.value_mut().updated_at = Some(now);
    drop(user);

    Ok(StatusCode::OK.into_response())
}
