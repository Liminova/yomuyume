use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use redb::ReadableDatabase;
use tracing::error;

use crate::{
    AppState,
    config::OperateMode,
    database,
    routes::{UserIDExtension, errors::InternalErr},
    utils::constants::CookieName,
};

/// Middleware checks `user-id` and `session-secret` cookies validity.
pub async fn auth(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, InternalErr> {
    req.extensions_mut().insert(UserIDExtension(None));

    let Some(provided_user_id) = cookie_jar
        .get(CookieName::UserID.as_ref())
        .map(|c| c.value_trimmed().to_string())
    else {
        if app_state.config.operate_mode == OperateMode::Public {
            return Ok(next.run(req).await);
        }
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let Some(provided_session_secret) = cookie_jar
        .get(CookieName::SessionSecret.as_ref())
        .map(|c| c.value_trimmed().to_string())
    else {
        if app_state.config.operate_mode == OperateMode::Public {
            return Ok(next.run(req).await);
        }
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    if app_state
        .db
        .user
        .begin_read()
        .map_err(|e| {
            error!("can't begin read transaction: {e:?}");
            InternalErr::DBTransactionError(e)
        })?
        .open_multimap_table(database::user::SESSIONS)
        .map_err(|e| {
            error!("can't open sessions table: {e:?}");
            InternalErr::DBTableError(e)
        })?
        .get(&provided_user_id)
        .map_err(|e| {
            error!("can't query sessions table: {e:?}");
            InternalErr::DBStorageError(e)
        })?
        .find(|s| match s.as_ref().map(|s| s.value()) {
            Ok(s) => s.session_secret == provided_session_secret,
            Err(e) => {
                error!("can't read session entry: {e:?}");
                false
            }
        })
        .is_some()
    {
        req.extensions_mut()
            .insert(UserIDExtension(Some(provided_user_id)));

        return Ok(next.run(req).await);
    };

    if app_state.config.operate_mode == OperateMode::Public {
        Ok(next.run(req).await)
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}
