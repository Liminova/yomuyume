use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::{
    models::{prelude::*, session_tokens::SessionSecret},
    routes::check_pass,
    AppError, AppState,
};

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct LoginRequestBody {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
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
    let user = match Users::find()
        .filter(users::Column::Username.eq(&query.login))
        .one(&app_state.db)
        .await
        .map_err(AppError::from)?
    {
        Some(user) => user,
        None => {
            return Ok((StatusCode::BAD_REQUEST, "invalid username or password").into_response())
        }
    };

    if !check_pass(&user.password, &query.password) {
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

    session_tokens::ActiveModel {
        session_secret: Set(session_secret.clone()),
        user_id: Set(user.id),
        created_at: Set(chrono::Utc::now()),
        user_agent: Set(user_agent),
        ip_address: Set(ip_address.clone()),
    }
    .insert(&app_state.db)
    .await
    .map_err(|e| AppError::from(anyhow::anyhow!("can't insert session token: {}", e)))?;

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
