use std::sync::Arc;

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use tracing::error;

use crate::{
    AppState,
    routes::errors::InternalError,
    utils::{
        constants::{CookieName, LOGOUT_PATH},
        result_utils::ResultUtils,
    },
};

/// Logout
#[utoipa::path(
    get,
    path = LOGOUT_PATH,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("user-id" = [], "session-secret" = [])))
]
pub async fn get_logout(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, InternalError> {
    if let Some(provided_user_id) = cookie_jar
        .get(CookieName::UserID.as_ref())
        .map(|c| c.value_trimmed().to_string())
        && let Some(provided_session_secret) = cookie_jar
            .get(CookieName::SessionSecret.as_ref())
            .map(|c| c.value_trimmed().to_string())
    {
        sqlx::query!(
            "DELETE FROM sessions WHERE user_id = ? AND secret = ?",
            provided_user_id,
            provided_session_secret
        )
        .execute(&app_state.pool)
        .await
        .log_err(|e| error!("can't delete session: {e}"))?;
    }

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (
                header::SET_COOKIE,
                Cookie::build((CookieName::UserID.as_ref(), ""))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .to_string(),
            ),
            (
                header::SET_COOKIE,
                Cookie::build((CookieName::SessionSecret.as_ref(), ""))
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
