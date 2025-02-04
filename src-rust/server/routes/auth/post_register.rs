use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::{
        errors::{InternalErr, RequestErr},
        hash_pass, is_strong,
    },
    utils::{app_state::AppState, constants::REGISTER_PATH},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Register
#[utoipa::path(post, path = REGISTER_PATH, responses(
    (status = 200, description = "Registration successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 409, description = "A conflict has occurred", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn post_register(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    query: Json<RegisterRequest>,
) -> Result<Response, InternalErr> {
    if !EmailAddress::is_valid(&query.email) {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidEmail).into_response());
    }

    if sqlx::query!(
        r#"SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE email = $1
            ) AS "exists!""#,
        query.email.to_string().to_ascii_lowercase()
    )
    .fetch_one(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        InternalErr::DB(e)
    })?
    .exists
    {
        return Ok((StatusCode::CONFLICT, RequestErr::SomeoneUseThisEmail).into_response());
    }

    if !is_strong(&query.password) {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::WeakPassword).into_response());
    }

    let ip_addr = app_state
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

    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, ip_address)
        VALUES ($1, $2, $3, $4, $5)",
        app_state.id_generator.snowflake().await.map_err(|e| {
            tracing::error!("{e:?}");
            InternalErr::Snowflake(e)
        })?,
        query.username.as_str(),
        query.email.to_string().to_ascii_lowercase(),
        hash_pass(query.password.as_bytes())?,
        ip_addr,
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        InternalErr::DB(e)
    })?;

    Ok(StatusCode::OK.into_response())
}
