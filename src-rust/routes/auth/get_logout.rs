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

use crate::{types::custom_id::SessionSecret, AppError, AppState};

/// Reset all the cookies on the client side.
#[utoipa::path(get, path = "/api/auth/logout", responses(
    (status = 200, description = "Logout successful"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_logout(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let session_secret = match cookie_jar
        .get("session-secret")
        .map(|cookie| cookie.value().to_string())
        .and_then(|raw| SessionSecret::from(raw).ok())
    {
        Some(session_secret) => session_secret,
        None => {
            return Ok(
                (StatusCode::UNAUTHORIZED, "no valid session secret provided").into_response(),
            )
        }
    };

    sqlx::query!(
        "DELETE FROM session_tokens WHERE session_secret = $1",
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
