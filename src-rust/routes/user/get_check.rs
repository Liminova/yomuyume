use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// check auth
///
/// check if the current session has vaid cookies
#[utoipa::path(get, path = "/api/user/check", responses(
    (status = 200, description = "cookies valid"),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_check() -> Response {
    (StatusCode::OK).into_response()
}
