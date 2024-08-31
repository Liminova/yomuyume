use std::sync::Arc;

use crate::{models::prelude::*, AppError, AppState};

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use sea_orm::*;
use session_tokens::SessionSecret;

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let session_id = match cookie_jar
        .get("session-id")
        .map(|cookie| cookie.value().to_string())
        .and_then(|session_id_str| session_id_str.parse::<u64>().ok())
    {
        Some(session_id) => session_id,
        None => {
            return Ok((StatusCode::UNAUTHORIZED, "no valid session id provided").into_response())
        }
    };

    let session_secret = match cookie_jar
        .get("session-secret")
        .map(|cookie| cookie.value().to_string())
        .and_then(|raw| SessionSecret::from(raw).ok())
    {
        Some(session_secret) => session_secret,
        None => {
            return Ok(
                (StatusCode::UNAUTHORIZED, "no valid session secret provided").into_response(),
            )
        }
    };

    let session_token = match SessionTokens::find_by_id(session_id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find session token: {}", e)))?
    {
        Some(session_token) => session_token,
        _ => return Ok((StatusCode::UNAUTHORIZED, "session token not found").into_response()),
    };

    if session_token.session_secret != session_secret {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    let user = Users::find_by_id(session_token.user_id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find user: {}", e)))?
        .ok_or({
            SessionTokens::delete_by_id(session_id)
                .exec(&data.db)
                .await
                .map_err(|e| {
                    AppError::from(anyhow::anyhow!("can't delete old session token: {}", e))
                })?;

            AppError::from(anyhow::anyhow!(
                "the user belonging to this session token no longer exists"
            ))
        })?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
