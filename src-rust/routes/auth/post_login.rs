use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{routes::check_pass, types::custom_id::SessionSecret, AppError, AppState};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequestBody {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponseBody {
    pub token: String,
}

/// Login with username and password and get the JWT token.
#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "Login successful"),
    (status = 500, description = "Internal server error", body = String),
    (status = 400, description = "Bad request", body = String),
))]
pub async fn post_login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    query: Json<LoginRequestBody>,
) -> Result<Response, AppError> {
    let (user_id, password_hash) = match sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1",
        query.login
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't find user")?
    {
        Some(user) => (user.id, user.password_hash),
        None => {
            return Ok((StatusCode::BAD_REQUEST, "invalid username or password").into_response());
        }
    };

    if !check_pass(&password_hash, &query.password) {
        return Ok((StatusCode::BAD_REQUEST, "invalid username or password").into_response());
    }

    let ip_address = app_state
        .config
        .reverse_proxy_ip_header
        .clone()
        .and_then(|header| {
            headers
                .get(&header)
                .or(headers.get(header.to_ascii_lowercase()))
        })
        .and_then(|value| value.to_str().ok())
        .map(|ip_str| ip_str.to_string())
        .unwrap_or(addr.ip().to_string());

    let user_agent = headers
        .get("user-agent")
        .or(headers.get("User-Agent"))
        .and_then(|value| value.to_str().ok())
        .map(|user_agent_str| user_agent_str.to_string());

    let session_secret = SessionSecret::new();

    sqlx::query!(
        "INSERT INTO session_tokens
            (session_secret, user_id, created_at, user_agent, ip_address, last_used_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (session_secret) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            created_at = EXCLUDED.created_at,
            user_agent = EXCLUDED.user_agent,
            ip_address = EXCLUDED.ip_address,
            last_used_at = EXCLUDED.last_used_at",
        session_secret.as_str(),
        user_id.as_str(),
        chrono::Utc::now(),
        user_agent,
        ip_address.as_str(),
        chrono::Utc::now()
    )
    .execute(&app_state.pool)
    .await
    .context("can't insert session token")?;

    let session_secret_cookie = Cookie::build(("session-secret", session_secret.to_string()))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, session_secret_cookie.to_string())],
    )
        .into_response())
}
