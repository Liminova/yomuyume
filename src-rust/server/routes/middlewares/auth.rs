use crate::{
    models::{auth::TokenClaims, prelude::*},
    routes::ErrRsp,
    AppState,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::*;
use std::sync::Arc;

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ErrRsp> {
    let token = cookie_jar
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
        })
        .ok_or_else(ErrRsp::no_token)?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| ErrRsp::no_token())?
    .claims;

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| ErrRsp::no_token())?;

    let user: Option<users::Model> = Users::find_by_id(user_id)
        .one(&data.db)
        .await
        .map_err(ErrRsp::db)?;

    if let Some(user) = user {
        req.extensions_mut().insert(user);
        req.extensions_mut()
            .insert(claims.purpose.unwrap_or_default());
        return Ok(next.run(req).await);
    }
    Err(ErrRsp::new(
        StatusCode::UNAUTHORIZED,
        "The user belonging to this token no longer exists.",
    ))
}
