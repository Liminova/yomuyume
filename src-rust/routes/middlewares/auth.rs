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
use chrono::{Duration, Utc};

use crate::{
    traits::chrono_utils::ChronoUtils,
    types::user_cache::UserCache,
    utils::{
        app_error::AppError,
        app_state::AppState,
        config::{SESSION_TOKEN_EXPIRED_AFTER, SESSION_TOKEN_UPDATE_LAST_USED_AT_INTERVAL},
    },
};

/// A middleware that checks `session-id` and `session-secret` cookies.
///
/// Added possible response codes:
/// - [`StatusCode::UNAUTHORIZED`]: if the cookies are invalid.
/// - [`StatusCode::INTERNAL_SERVER_ERROR`]: if there's an error while querying
///   the database to check the cookies.
pub async fn auth(
    cookie_jar: CookieJar,
    State(app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // preparing variables
    let Some(provided_session_id) = cookie_jar
        .get("session-id")
        .map(|c| c.to_string())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|i| i.into())
    else {
        return Ok((StatusCode::UNAUTHORIZED, "missing session id").into_response());
    };
    let Some(provided_session_secret) = cookie_jar.get("session-secret").map(|c| c.to_string())
    else {
        return Ok((StatusCode::UNAUTHORIZED, "missing session secret").into_response());
    };
    let now = Utc::now();

    // fetch using id from mem/db
    let (user_id, ss_token_last_used_at, session_secret) = 'scoped: {
        if let Some(user_id) = app_state.session_cache.get(&provided_session_id) {
            if let Some(session_token) = app_state.user_cache.get(&user_id) {
                break 'scoped (
                    *user_id,
                    session_token.ss_token_last_used_at,
                    session_token.session_secret.clone(),
                );
            }
        }

        if let Some(r) = sqlx::query!(
            "SELECT u.id AS user_id,
                u.username AS username,
                u.email AS email,
                u.profile_picture AS profile_picture,
                u.ip_address AS ip_address,
                u.updated_at AS updated_at,
                u.verified_at AS verified_at,
                s.last_used_at,
                s.session_secret
            FROM session_tokens AS s
                JOIN users AS u ON s.user_id = u.id
            WHERE s.id = $1
                AND s.session_secret = $2",
            provided_session_id.as_ref(),
            provided_session_secret.as_str()
        )
        .fetch_optional(&app_state.pool)
        .await
        .context("can't find user")?
        {
            let new_user_cache = UserCache {
                username: r.username,
                email: r.email,
                profile_picture: r.profile_picture,
                ip_address: r.ip_address,
                updated_at: r.updated_at,
                verified_at: r.verified_at,
                session_secret: r.session_secret,
                ss_token_last_used_at: r.last_used_at.unwrap_or(now),
            };

            let user_id = r.user_id.into();
            let ss_token_last_used_at = new_user_cache.ss_token_last_used_at;
            let session_secret = new_user_cache.session_secret.clone();

            app_state.user_cache.insert(user_id, new_user_cache);
            app_state.session_cache.insert(provided_session_id, user_id);

            break 'scoped (user_id, ss_token_last_used_at, session_secret);
        }

        return Ok((StatusCode::UNAUTHORIZED, "session token not found").into_response());
    };

    // validate
    if provided_session_secret != session_secret {
        return Ok((StatusCode::UNAUTHORIZED, "invalid session secret").into_response());
    }

    // expired
    if ss_token_last_used_at.outside(&now, &Duration::seconds(SESSION_TOKEN_EXPIRED_AFTER)) {
        app_state.session_cache.remove(&provided_session_id);

        sqlx::query!(
            "DELETE FROM session_tokens
            WHERE id = $1",
            provided_session_id.as_ref()
        )
        .execute(&app_state.pool)
        .await
        .context("can't delete expired session token")?;

        return Ok((StatusCode::UNAUTHORIZED, "session token expired").into_response());
    }

    // update last used at if needed
    if ss_token_last_used_at.outside(
        &now,
        &Duration::seconds(SESSION_TOKEN_UPDATE_LAST_USED_AT_INTERVAL),
    ) {
        sqlx::query!(
            "UPDATE session_tokens
            SET last_used_at = $1
            WHERE id = $2",
            now,
            provided_session_id.as_ref()
        )
        .execute(&app_state.pool)
        .await
        .context("can't update session token's last used time")?;

        app_state
            .user_cache
            .get_mut(&user_id)
            .context("can't find user in cache, this should not happend")?
            .value_mut()
            .ss_token_last_used_at = now;
    }

    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}
