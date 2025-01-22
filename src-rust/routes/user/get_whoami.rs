use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::{
    app_error::AppError, app_state::AppState, config::SESSION_TOKEN_LAST_USED_AT_UPDATE_INTERVAL,
};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct WhoAmIResponseBody {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub ip_address: String,
    pub updated_at: Option<String>,
    pub verified_at: Option<String>,
}

/// whoami
///
/// get the current user's information
#[utoipa::path(get, path = "/api/user/whoami", responses(
    (status = 200, description = "get whoami successful", body = WhoAmIResponseBody),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_whoami(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let Some(session_id) = cookie_jar
        .get("session-id")
        .and_then(|cookie| cookie.value().to_string().parse::<i64>().ok())
    else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let Some(session_secret) = cookie_jar
        .get("session-secret")
        .map(|cookie| cookie.value().to_string())
    else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let Some(record) = sqlx::query!(
        "SELECT
            users.id as user_id,
            users.username,
            users.email,
            users.profile_picture,
            users.ip_address,
            users.updated_at,
            users.verified_at,
            session_tokens.last_used_at
        FROM session_tokens
            JOIN users ON session_tokens.user_id = users.id
        WHERE session_tokens.id = $1
            AND session_tokens.session_secret = $2
            AND users.id = session_tokens.user_id",
        session_id,
        session_secret.as_str()
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't find user")?
    else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let now = Utc::now();
    if (now - record.last_used_at.unwrap_or(now)).num_seconds()
        > SESSION_TOKEN_LAST_USED_AT_UPDATE_INTERVAL
    {
        sqlx::query!(
            "UPDATE session_tokens SET last_used_at = $1 WHERE id = $2",
            now,
            session_id
        )
        .execute(&app_state.pool)
        .await
        .context("can't update session token's last used time")?;
    }

    Ok((
        StatusCode::OK,
        Json(WhoAmIResponseBody {
            user_id: record.user_id,
            username: record.username,
            email: record.email,
            profile_picture: record.profile_picture,
            ip_address: record.ip_address,
            updated_at: record.updated_at.map(|d| d.to_rfc3339()),
            verified_at: record.verified_at.map(|v| v.to_string()),
        }),
    )
        .into_response())
}
