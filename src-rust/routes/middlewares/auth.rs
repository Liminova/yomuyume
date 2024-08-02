use std::sync::Arc;

use crate::{
    models::{auth::TokenClaims, prelude::*},
    AppError, AppState,
};

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::*;

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let token = match cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|stripped| stripped.to_owned())
                })
        }) {
        Some(token) => token.to_string(),
        None => return Ok((StatusCode::UNAUTHORIZED, "No token provided").into_response()),
    };

    let claims = match decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(data.config.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(claims) => claims,
        Err(e) => return Ok((StatusCode::UNAUTHORIZED, e.to_string()).into_response()),
    }
    .claims;

    let user_id_string: String = claims
        .sub
        .parse()
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't parse user id: {}", e)))?;

    let user_id = UserID::from(user_id_string)
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't parse user id: {}", e)))?;

    match Users::find_by_id(user_id)
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find user: {}", e)))?
    {
        Some(user) => {
            req.extensions_mut().insert(user);
            req.extensions_mut()
                .insert(claims.purpose.unwrap_or_default());
            Ok(next.run(req).await)
        }
        None => Ok((
            StatusCode::UNAUTHORIZED,
            "The user belonging to this token no longer exists.",
        )
            .into_response()),
    }
}
