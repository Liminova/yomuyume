use std::sync::Arc;

use axum::{
    Json,
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
        errors::{InternalError, RequestErr},
        hash_pass, is_strong,
    },
    utils::{constants::REGISTER_PATH, nanoid::nanoid, result_utils::ResultUtils},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: Option<String>,
    pub email: String,
    pub password: String,
}

/// Register
#[utoipa::path(
    post,
    path = REGISTER_PATH,
    responses(
        (status = 200, description = "Registration successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 409, description = "A conflict has occurred", body = String),
        (status = 500, description = "Internal server error", body = String),
    ))
]
pub async fn post_register(
    State(app_state): State<Arc<AppState>>,
    query: Json<RegisterRequest>,
) -> Result<Response, InternalError> {
    let Ok(email) = query
        .email
        .parse::<lettre::Address>()
        .map(|e| e.to_string())
    else {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidEmail).into_response());
    };

    if sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?) AS `exists`",
        email
    )
    .fetch_one(&app_state.pool)
    .await
    .map(|r| r.exists == 1)
    .inspect_err(|e| error!("can't query user by email: {e}"))?
    {
        return Ok((StatusCode::CONFLICT, RequestErr::EmailAlreadyUsed).into_response());
    }

    if !is_strong(&query.password) {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::WeakPassword).into_response());
    }

    let password_hash = hash_pass(&query.password).map_err(|e| {
        error!("can't hash password: {}", e);
        InternalError::PasswordHash(e)
    })?;

    let user_id = nanoid();
    sqlx::query!(
        "INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)",
        user_id,
        email,
        password_hash
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't insert new user: {e}"))?;

    Ok(StatusCode::OK.into_response())
}
