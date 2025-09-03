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
use redb::ReadableDatabase;
use tracing::error;

use crate::{
    AppState, database,
    routes::errors::InternalErr,
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
) -> Result<Response, InternalErr> {
    if let Some(provided_user_id) = cookie_jar
        .get(CookieName::UserID.as_ref())
        .map(|c| c.value_trimmed().to_string())
        && let Some(provided_session_secret) = cookie_jar
            .get(CookieName::SessionSecret.as_ref())
            .map(|c| c.value_trimmed().to_string())
        && let Some(session) = app_state
            .db
            .user
            .begin_read()
            .log_err(|e| error!("can't begin read transaction: {e}"))?
            .open_multimap_table(database::user::SESSIONS)
            .log_err(|e| error!("can't open sessions table: {e}"))?
            .get(&provided_user_id)
            .log_err(|e| error!("can't get session: {e}"))?
            .find_map(|r| match r.map(|r| r.value()) {
                Ok(session) if session.session_secret == provided_session_secret => Some(session),
                Ok(_) => None,
                Err(e) => {
                    error!("can't read session: {e}");
                    None
                }
            })
    {
        let write_txn = app_state
            .db
            .user
            .begin_write()
            .log_err(|e| error!("can't begin write transaction: {e}"))?;

        write_txn
            .open_multimap_table(database::user::SESSIONS)
            .log_err(|e| error!("can't open sessions table: {e}"))?
            .remove(&provided_user_id, &session)
            .log_err(|e| error!("can't remove session: {e}"))?;

        write_txn
            .commit()
            .log_err(|e| error!("can't commit write transaction: {e}"))?;
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
