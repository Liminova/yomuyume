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
    utils::constants::{LOGOUT_PATH, SESSION_SECRET_COOKIE_NAME},
};

/// Logout
#[utoipa::path(
    get,
    path = LOGOUT_PATH,
    responses(
        (status = 200, description = "Logout successful"),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("session-secret" = []))
)]
pub async fn get_logout(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, InternalError> {
    if let Some(provided_session_secret) = cookie_jar
        .get(SESSION_SECRET_COOKIE_NAME)
        .map(|c| c.value_trimmed().to_string())
    {
        sqlx::query!(
            "DELETE FROM sessions WHERE secret = ?",
            provided_session_secret
        )
        .execute(&app_state.pool)
        .await
        .inspect_err(|e| error!("can't delete session: {e}"))?;
    }

    Ok((
        StatusCode::OK,
        AppendHeaders([(
            header::SET_COOKIE,
            Cookie::build((SESSION_SECRET_COOKIE_NAME, ""))
                .path("/")
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Strict)
                .to_string(),
        )]),
    )
        .into_response())
}
