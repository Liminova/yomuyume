use std::sync::Arc;

pub use bridge::routes::auth::{LoginRequest, LoginResponseBody};

use crate::{
    models::{auth::TokenClaims, prelude::*},
    routes::{check_pass, MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct LoremIpsum {
    pub ipsum: String,
}

/// Login with username and password and get the JWT token.
#[utoipa::path(post, path = "/api/auth/login", responses(
    (status = 200, description = "Login successful", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
))]
pub async fn post_login(
    State(data): State<Arc<AppState>>,
    header: HeaderMap,
    query: Json<LoginRequest>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let user: users::Model = Users::find()
        .filter(users::Column::Username.eq(&query.login))
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.bad_request("Invalid username or password."))?;

    if !check_pass(&user.password, &query.password) {
        return Err(builder.bad_request("Invalid username or password."));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + data.env.jwt_maxage_day).timestamp() as usize;
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
    .map_err(|e| builder.internal(format!("JWT error: {}", e)))?;

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::days(data.env.jwt_maxage_day.num_days()))
        .same_site(SameSite::Lax)
        .http_only(true);

    Ok(builder
        .success(LoginResponseBody { token })
        .add_header((header::SET_COOKIE, cookie.to_string())))
}
