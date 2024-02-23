use crate::{
    models::{auth::TokenClaims, prelude::*},
    routes::{check_pass, ErrRsp},
    AppState,
};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponseBody {
    pub token: String,
}

/// Login with username and password and get the JWT token.
#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "Login successful", body = LoginResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
))]
pub async fn post_login(
    State(data): State<Arc<AppState>>,
    query: Json<LoginRequest>,
) -> Result<impl IntoResponse, ErrRsp> {
    let user: users::Model = Users::find()
        .filter(users::Column::Username.eq(&query.login))
        .one(&data.db)
        .await
        .map_err(ErrRsp::db)?
        .ok_or_else(|| ErrRsp::bad_request("Invalid username or password."))?;

    if !check_pass(&user.password, &query.password) {
        return Err(ErrRsp::bad_request("Invalid username or password."));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + data.env.jwt_maxage).timestamp() as usize;
    let claims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
        purpose: None,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .map_err(|e| ErrRsp::internal(format!("JWT error: {}", e)))?;

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::days(data.env.jwt_maxage.num_days()))
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(LoginResponseBody { token }),
    ))
}
