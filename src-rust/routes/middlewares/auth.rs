use std::sync::Arc;

use anyhow::Context;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use chrono::Utc;

use crate::{AppError, AppState};

/// A middleware that checks `session-id` and `session-secret` cookies.
///
/// Added possible response codes:
/// - [`StatusCode::UNAUTHORIZED`]: if the cookies are invalid.
/// - [`StatusCode::INTERNAL_SERVER_ERROR`]: if there's an error while querying
///   the database to check the cookies.
pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let session_id = match cookie_jar
        .get("session-id")
        .and_then(|cookie| cookie.value().to_string().parse::<i64>().ok())
    {
        Some(session_id) => session_id,
        None => {
            return Ok((StatusCode::UNAUTHORIZED, "no valid session id provided").into_response())
        }
    };

    let session_secret = match cookie_jar
        .get("session-secret")
        .map(|cookie| cookie.value().to_string())
    {
        Some(session_secret) => session_secret,
        None => {
            return Ok(
                (StatusCode::UNAUTHORIZED, "no valid session secret provided").into_response(),
            )
        }
    };

    let (user_id, last_used_at) = match sqlx::query!(
        "SELECT users.id, session_tokens.last_used_at FROM session_tokens
        JOIN users ON session_tokens.user_id = users.id
        WHERE session_tokens.id = $1 AND session_tokens.session_secret = $2",
        session_id,
        session_secret.as_str()
    )
    .fetch_optional(&data.pool)
    .await
    .context("can't find user")?
    {
        Some(result) => (result.id, result.last_used_at.unwrap_or(Utc::now())),
        None => {
            return Ok((StatusCode::UNAUTHORIZED, "session token not found").into_response());
        }
    };

    let now = Utc::now();
    if now - last_used_at < chrono::Duration::minutes(2) {
        sqlx::query!(
            "UPDATE session_tokens SET last_used_at = $1 WHERE id = $2",
            now,
            session_id
        )
        .execute(&data.pool)
        .await
        .context("can't update session token's last used time")?;
    }

    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}
