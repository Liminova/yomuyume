use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};

/// Reset all the cookies on the client side.
#[utoipa::path(get, path = "/api/auth/logout", responses(
    (status = 200, description = "Logout successful"),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn get_logout() -> Response {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    (StatusCode::OK, [(header::SET_COOKIE, cookie.to_string())]).into_response()
}
