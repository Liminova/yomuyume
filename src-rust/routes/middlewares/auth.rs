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

    let session_token = match SessionTokens::find_by_id(&session_secret)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find session token: {}", e)))?
    {
        Some(session_token) => session_token,
        _ => return Ok((StatusCode::UNAUTHORIZED, "session token not found").into_response()),
    };

    let user_model = Users::find_by_id(&session_token.user_id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find user: {}", e)))?
        .ok_or({
            SessionTokens::delete_by_id(session_secret)
                .exec(&data.db)
                .await
                .map_err(|e| {
                    AppError::from(anyhow::anyhow!("can't delete old session token: {}", e))
                })?;

            AppError::from(anyhow::anyhow!(
                "the user belonging to this session token no longer exists"
            ))
        })?;

    let last_used_at = user_model
        .last_used_at
        .clone()
        .unwrap_or("1970-01-01 00:00:00".parse().unwrap_or_default());
    let mut user_active_model: users::ActiveModel = user_model.clone().into();
    let now = chrono::Utc::now();
    if now - last_used_at < chrono::Duration::minutes(2) {
        user_active_model.last_used_at = Set(Some(chrono::Utc::now()));
        user_active_model
            .update(&data.db)
            .await
            .context("can't update last_used_at")?;
    }

    req.extensions_mut().insert(user_model);
    Ok(next.run(req).await)
}
