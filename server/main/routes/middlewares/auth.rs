use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use tracing::error;

use crate::{
    AppState,
    config::OperateMode,
    routes::{UserIDExtension, errors::InternalError},
    utils::{constants::CookieName, result_utils::ResultUtils},
};

/// Middleware checks `user-id` and `session-secret` cookies validity.
pub async fn auth(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, InternalError> {
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

    if let Some(user) = sqlx::query!(
        "SELECT user_id FROM sessions WHERE user_id = ? AND secret = ?",
        provided_user_id,
        provided_session_secret
    )
    .fetch_optional(&app_state.pool)
    .await
    .inspect_err(|e| error!("failed to query sessions: {e}"))?
    {
        req.extensions_mut()
            .insert(UserIDExtension(Some(user.user_id)));
        return Ok(next.run(req).await);
    };

    if app_state.config.operate_mode == OperateMode::Public {
        Ok(next.run(req).await)
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}
