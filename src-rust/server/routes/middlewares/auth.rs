use std::sync::Arc;

use crate::{
    models::{auth::TokenClaims, prelude::*},
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Request},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::*;

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    header: HeaderMap,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

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
        .ok_or_else(|| builder.unauthorized("No token provided"))?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| builder.unauthorized("Invalid token"))?
    .claims;

    let user_id: String = claims
        .sub
        .parse()
        .map_err(|_| builder.unauthorized("Invalid token"))?;

    let user: Option<users::Model> = Users::find_by_id(user_id)
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    if let Some(user) = user {
        req.extensions_mut().insert(user);
        req.extensions_mut()
            .insert(claims.purpose.unwrap_or_default());
        return Ok(next.run(req).await);
    }

    Ok(builder
        .unauthorized("The user belonging to this token no longer exists.")
        .into_response())
}
