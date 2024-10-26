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
use sea_orm::EntityTrait;

use crate::{
    models::{prelude::*, session_tokens::SessionSecret},
    AppError, AppState,
};

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

    let _ = SessionTokens::delete_by_id(session_secret)
        .exec(&app_state.db)
        .await
        .context("can't delete session token")?;

    let cookie = Cookie::build(("token", ""))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie.to_string())]).into_response())
}
