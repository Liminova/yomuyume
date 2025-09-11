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
    routes::{
        UserIDExtension,
        errors::{InternalError, RequestError},
    },
    utils::constants::SESSION_SECRET_COOKIE_NAME,
};

/// Middleware checks `user-id` and `session-secret` cookies validity.
pub async fn auth(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, InternalError> {
    req.extensions_mut().insert(UserIDExtension(None));

    let Some(provided_session_secret) = cookie_jar
        .get(SESSION_SECRET_COOKIE_NAME)
        .map(|c| c.value_trimmed().to_string())
    else {
        if app_state.config.operate_mode == OperateMode::Public {
            return Ok(next.run(req).await);
        }
        return Ok((StatusCode::UNAUTHORIZED, RequestError::MissingSessionSecret).into_response());
    };

    if let Some(user) = sqlx::query!(
        "SELECT user_id FROM sessions WHERE secret = ?",
        provided_session_secret
    )
    .fetch_optional(&app_state.pool)
    .await
    .inspect_err(|e| error!("failed to query sessions: {e}"))?
    {
        req.extensions_mut()
            .insert(UserIDExtension(Some(user.user_id)));
        return Ok(next.run(req).await);
    }

    if app_state.config.operate_mode == OperateMode::Public {
        Ok(next.run(req).await)
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}
