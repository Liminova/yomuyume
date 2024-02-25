use crate::routes::MyResponseBuilder;

use axum::{
    http::{header, HeaderMap},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

/// Reset all the cookies on the client side.
#[utoipa::path(get, path = "/api/auth/logout", responses(
    (status = 200, description = "Logout successful", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
))]
pub async fn get_logout(header: HeaderMap) -> impl IntoResponse {
    let builder = MyResponseBuilder::new(header);

    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    builder
        .generic_success("Logout successful.")
        .add_header((header::SET_COOKIE, cookie.to_string()))
}
