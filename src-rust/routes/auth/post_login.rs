use std::sync::Arc;

use crate::{
    models::{auth::TokenClaims, prelude::*},
    routes::check_pass,
    AppError, AppState,
};

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct LoginRequestBody {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct LoginResponseBody {
    pub token: String,
}

/// Login with username and password and get the JWT token.
#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "Login successful", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = String),
    (status = 400, description = "Bad request", body = String),
))]
pub async fn post_login(
    State(data): State<Arc<AppState>>,
    query: Json<LoginRequestBody>,
) -> Result<Response, AppError> {
    // get user from db
    let user = match Users::find()
        .filter(users::Column::Username.eq(&query.login))
        .one(&data.db)
        .await
        .map_err(AppError::from)?
    {
        Some(user) => user,
        None => {
            return Ok((StatusCode::BAD_REQUEST, "Invalid username or password.").into_response())
        }
    };

    // check password
    if !check_pass(&user.password, &query.password) {
        return Ok((StatusCode::BAD_REQUEST, "Invalid username or password.").into_response());
    }

    // build claims
    let now = chrono::Utc::now();
    let claims = TokenClaims {
        sub: user.id.to_string(),
        exp: (now + data.config.jwt_maxage_day).timestamp() as usize,
        iat: now.timestamp() as usize,
        purpose: None,
    };

    // generate token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.config.jwt_secret.as_ref()),
    )
    .map_err(AppError::from)?;

    // build cookie
    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::days(data.config.jwt_maxage_day.num_days()))
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(LoginResponseBody { token }),
    )
        .into_response())
}
