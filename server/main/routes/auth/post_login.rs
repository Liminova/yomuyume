use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    AppState,
    routes::{check_pass, errors::InternalError},
    utils::{
        constants::{CookieName, LOGIN_PATH},
        nanoid::nanoid,
        result_utils::ResultUtils,
    },
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login
#[utoipa::path(
    post,
    path = LOGIN_PATH,
    responses(
        (status = 200, description = "Login successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String),
    ))
]
pub async fn post_login(
    State(app_state): State<Arc<AppState>>,
    query: Json<LoginRequest>,
) -> Result<Response, InternalError> {
    let Some((user_id, password_hash)) = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE email = ?",
        query.email
    )
    .fetch_optional(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't query user by email: {e}"))?
    .map(|r| (r.id, r.password_hash)) else {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
    };

    if !check_pass(&password_hash, &query.password) {
        return Ok((StatusCode::BAD_REQUEST, "invalid password").into_response());
    }

    let session_secret = nanoid();
    sqlx::query!(
        "INSERT INTO sessions (secret, user_id) VALUES (?, ?)",
        session_secret,
        user_id
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't create session: {e}"))?;

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (
                header::SET_COOKIE,
                Cookie::build((CookieName::UserID.as_ref(), user_id))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .to_string(),
            ),
            (
                header::SET_COOKIE,
                Cookie::build((CookieName::SessionSecret.as_ref(), session_secret))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .to_string(),
            ),
        ]),
    )
        .into_response())
}
