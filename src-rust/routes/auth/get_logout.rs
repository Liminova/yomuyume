use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};

use crate::{AppError, AppState};

/// logout
///
/// reset all the cookies on the client side
#[utoipa::path(get, path = "/api/auth/logout", responses(
    (status = 200, description = "logout successful"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_logout(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let session_id = match cookie_jar
        .get("session-id")
        .and_then(|cookie| cookie.value().to_string().parse::<i64>().ok())
    {
        Some(session_id) => session_id,
        None => {
            return Ok((StatusCode::UNAUTHORIZED, "no valid session id provided").into_response())
        }
    };

    let session_secret = match cookie_jar
        .get("session-secret")
        .map(|cookie| cookie.value().to_string())
    {
        Some(session_secret) => session_secret,
        None => {
            return Ok(
                (StatusCode::UNAUTHORIZED, "no valid session secret provided").into_response(),
            )
        }
    };

    sqlx::query!(
        "DELETE FROM session_tokens WHERE id = $1 AND session_secret = $2",
        session_id,
        session_secret.as_str()
    )
    .execute(&app_state.pool)
    .await
    .context("can't delete session token")?;

    let cookie = Cookie::build(("token", ""))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie.to_string())]).into_response())
}
