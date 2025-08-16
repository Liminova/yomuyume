use std::sync::Arc;

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
    routes::errors::{InternalErr, RequestErr},
    traits::chrono_utils::ChronoUtils,
    utils::{
        app_state::AppState,
        cacher::{CachedSessionBuilder, CachedUserBuilder},
        constants::{
            SESSION_EXPIRED_AFTER, SESSION_ID_COOKIE_NAME, SESSION_SECRET_COOKIE_NAME,
            SESSION_UPDATE_LAST_USED_AT_IF_OLDER_THAN_SECONDS,
        },
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
) -> Result<Response, InternalErr> {
    let provided_session_id = match cookie_jar
        .get(SESSION_ID_COOKIE_NAME)
        .ok_or(RequestErr::MissingSessionID)
        .map(|c| c.value_trimmed().to_string())
        .and_then(|s| s.parse::<i64>().map_err(RequestErr::CantParseSessionID))
    {
        Ok(i) => i.into(),
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e).into_response()),
    };
    let Some(provided_session_secret) = cookie_jar
        .get(SESSION_SECRET_COOKIE_NAME)
        .map(|c| c.value_trimmed().to_string())
    else {
        return Ok((StatusCode::UNAUTHORIZED, RequestErr::MissingSessionSecret).into_response());
    };

    // fetch using id from mem/db
    let (user_id, session_last_used_at, session_secret) = 'scoped: {
        if let Some(cached_session) = app_state.cacher.sessions.get(&provided_session_id) {
            break 'scoped (
                cached_session.user_id,
                cached_session.last_used_at,
                cached_session.secret.clone(),
            );
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
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::DB(e)
        })? {
            let new_cached_user = CachedUserBuilder::default()
                .username(r.username)
                .email(r.email)
                .profile_picture(r.profile_picture)
                .ip_address(r.ip_address)
                .updated_at(r.updated_at)
                .verified_at(r.verified_at)
                .build();

            let user_id = r.user_id.into();
            let session_last_used_at = r.last_used_at;
            let session_secret = r.session_secret;

            app_state.cacher.users.insert(user_id, new_cached_user);
            app_state.cacher.sessions.insert(
                provided_session_id,
                CachedSessionBuilder::default()
                    .user_id(user_id)
                    .secret(session_secret.clone())
                    .last_used_at(session_last_used_at)
                    .build(),
            );

            break 'scoped (user_id, session_last_used_at, session_secret);
        }

        return Ok((StatusCode::UNAUTHORIZED, RequestErr::InvalidSession).into_response());
    };

    if provided_session_secret != session_secret {
        return Ok((StatusCode::UNAUTHORIZED, RequestErr::InvalidSession).into_response());
    }

    let now = Utc::now();
    let token_expired =
        session_last_used_at.outside(&now, &Duration::seconds(SESSION_EXPIRED_AFTER));
    if token_expired {
        app_state.cacher.sessions.remove(&provided_session_id);
        sqlx::query!(
            "DELETE FROM session_tokens
            WHERE id = $1",
            provided_session_id.as_ref()
        )
        .execute(&app_state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::DB(e)
        })?;

        return Ok((StatusCode::UNAUTHORIZED, RequestErr::SessionExpired).into_response());
    }

    let need_update_last_used_at = session_last_used_at.outside(
        &now,
        &Duration::seconds(SESSION_UPDATE_LAST_USED_AT_IF_OLDER_THAN_SECONDS),
    );
    if need_update_last_used_at {
        sqlx::query!(
            "UPDATE session_tokens
            SET last_used_at = $1
            WHERE id = $2",
            now,
            provided_session_id.as_ref()
        )
        .execute(&app_state.pool)
        .await
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::DB(e)
        })?;

        app_state
            .cacher
            .sessions
            .get_mut(&provided_session_id)
            .ok_or_else(|| {
                let e = InternalErr::WriteCache("session".to_string());
                tracing::error!("{e}");
                e
            })?
            .value_mut()
            .last_used_at = now;
    }

    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}
