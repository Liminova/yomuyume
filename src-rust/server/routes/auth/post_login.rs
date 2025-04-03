use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    app_state::AppState,
    routes::{
        check_pass,
        errors::{InternalErr, RequestErr},
    },
    utils::constants::{LOGIN_PATH, SESSION_ID_COOKIE_NAME, SESSION_SECRET_COOKIE_NAME},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    /// username or email
    pub login: String,
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    query: Json<LoginRequest>,
) -> Result<Response, InternalErr> {
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
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?
    .map(|user| (user.id, user.password_hash)) else {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidCredentials).into_response());
    };

    if !check_pass(&password_hash, &query.password) {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidCredentials).into_response());
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

    let session_secret = app_state.id_generator.secure().map_err(|e| {
        tracing::error!("{e}");
        InternalErr::SecureID(e)
    })?;

    let session_id = sqlx::query!(
        "INSERT INTO session_tokens (
                id,
                session_secret,
                user_id,
                user_agent,
                ip_address,
                last_used_at
            )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id",
        app_state.id_generator.snowflake().await.map_err(|e| {
            tracing::error!("{e}");
            InternalErr::Snowflake(e)
        })?,
        session_secret.as_str(),
        user_id,
        user_agent,
        ip_address.as_str(),
        chrono::Utc::now(),
    )
    .fetch_one(&app_state.pool)
    .await
    .map(|r| r.id)
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (
                header::SET_COOKIE,
                Cookie::build((SESSION_ID_COOKIE_NAME, session_id.to_string()))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .to_string(),
            ),
            (
                header::SET_COOKIE,
                Cookie::build((SESSION_SECRET_COOKIE_NAME, session_secret))
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
