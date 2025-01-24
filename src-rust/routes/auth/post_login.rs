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

use crate::{app_state::AppState, routes::check_pass, utils::app_error::AppError};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequestBody {
    /// username or email
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponseBody {
    pub token: String,
}

/// login
#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "login successful"),
    (status = 400, description = "bad request", body = String),
    (status = 500, description = "internal server error", body = String),
))]
pub async fn post_login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    query: Json<LoginRequestBody>,
) -> Result<Response, AppError> {
    let Some((user_id, password_hash)) = sqlx::query!(
        "SELECT id,
            password_hash
        FROM users
        WHERE username = $1
            OR email = $1",
        query.login
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query user")?
    .map(|user| (user.id, user.password_hash)) else {
        return Ok((StatusCode::BAD_REQUEST, "invalid username or password").into_response());
    };

    if !check_pass(&password_hash, &query.password) {
        return Ok((StatusCode::BAD_REQUEST, "invalid username or password").into_response());
    }

    let ip_address = app_state
        .config
        .reverse_proxy_ip_header
        .as_ref()
        .and_then(|header| {
            headers
                .get(header)
                .or_else(|| headers.get(header.to_ascii_lowercase()))
        })
        .and_then(|value| value.to_str().ok())
        .map_or_else(|| addr.ip().to_string(), |ip_str| ip_str.to_string());

    let user_agent = headers
        .get("user-agent")
        .or_else(|| headers.get("User-Agent"))
        .and_then(|value| value.to_str().ok())
        .map(|user_agent_str| user_agent_str.to_string());

    let session_secret = app_state.id_generator.secure();

    sqlx::query!(
        "INSERT INTO session_tokens
            (id, session_secret, user_id, user_agent, ip_address, last_used_at)
        VALUES ($1, $2, $3, $4, $5, $6)",
        app_state
            .id_generator
            .snowflake()
            .await
            .context("can't generate ID for record")?,
        session_secret.as_str(),
        user_id,
        user_agent,
        ip_address.as_str(),
        chrono::Utc::now(),
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
