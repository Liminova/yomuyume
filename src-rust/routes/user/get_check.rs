use crate::GenericResponseBody;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

/// Check if the token in the request header/cookie is valid.
#[utoipa::path(get, path = "/api/user/check", responses(
    (status = 200, description = "Cookies valid.", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn get_check() -> Response {
    (
        StatusCode::OK,
        Json(GenericResponseBody::new("Cookies valid.")),
    )
        .into_response()
}
