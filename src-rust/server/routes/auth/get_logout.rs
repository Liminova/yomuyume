use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::routes::GenericResponseBody;

/// Reset all the cookies on the client side.
#[utoipa::path(get, path = "/api/auth/logout", responses(
    (status = 200, description = "Logout successful", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
))]
pub async fn get_logout() -> impl IntoResponse {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(GenericResponseBody {
            message: "Logout successful.".to_string(),
        }),
    )
}
