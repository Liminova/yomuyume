use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};

use crate::{
    routes::errors::{InternalError, RequestError},
    types::id::SessionID,
    utils::app_state::AppState,
};

/// Logout
#[utoipa::path(get, path = "/api/auth/logout", responses(
    (status = 200, description = "Logout successful"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_logout(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, InternalError> {
    let Some(session_id): Option<SessionID> = cookie_jar
        .get("session-id")
        .and_then(|cookie| cookie.to_string().parse::<i64>().ok())
        .map(|id| id.into())
    else {
        return Ok((StatusCode::UNAUTHORIZED, RequestError::YouDontEvenLoggedIn).into_response());
    };

    let Some(session_secret) = cookie_jar
        .get("session-secret")
        .map(|cookie| cookie.to_string())
    else {
        return Ok((StatusCode::UNAUTHORIZED, RequestError::YouDontEvenLoggedIn).into_response());
    };

    sqlx::query!(
        "DELETE FROM session_tokens WHERE id = $1 AND session_secret = $2",
        session_id.as_ref(),
        session_secret.as_str()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        InternalError::DB(e)
    })?;

    app_state.session_cache.remove(&session_id);

    let cookie = Cookie::build(("token", ""))
        .path("/")
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((StatusCode::OK, [(header::SET_COOKIE, cookie.to_string())]).into_response())
}
