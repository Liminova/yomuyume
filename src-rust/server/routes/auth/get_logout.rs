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
    routes::errors::{InternalErr, RequestErr},
    structs::ids::SessionID,
    utils::{
        app_state::AppState,
        constants::{LOGOUT_PATH, SESSION_ID_COOKIE_NAME, SESSION_SECRET_COOKIE_NAME},
    },
};

/// Logout
#[utoipa::path(
    get,
    path = LOGOUT_PATH,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-id" = [], "session-secret" = [])))
]
pub async fn get_logout(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, InternalErr> {
    let Some(session_id): Option<SessionID> = cookie_jar
        .get(SESSION_ID_COOKIE_NAME)
        .and_then(|cookie| cookie.to_string().parse::<i64>().ok())
        .map(|id| id.into())
    else {
        return Ok((StatusCode::UNAUTHORIZED, RequestErr::YouDontEvenLoggedIn).into_response());
    };

    let Some(session_secret) = cookie_jar
        .get(SESSION_SECRET_COOKIE_NAME)
        .map(|cookie| cookie.to_string())
    else {
        return Ok((StatusCode::UNAUTHORIZED, RequestErr::YouDontEvenLoggedIn).into_response());
    };

    sqlx::query!(
        "DELETE FROM session_tokens WHERE id = $1 AND session_secret = $2",
        session_id.as_ref(),
        session_secret.as_str()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    app_state.session_cache.remove(&session_id);

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (
                header::SET_COOKIE,
                Cookie::build((SESSION_ID_COOKIE_NAME, ""))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .to_string(),
            ),
            (
                header::SET_COOKIE,
                Cookie::build((SESSION_SECRET_COOKIE_NAME, ""))
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
