use axum::response::IntoResponse;

use crate::routes::GenericRsp;

/// Check if the cookies are valid.
#[utoipa::path(get, path = "/api/user/check", responses(
    (status = 200, description = "Cookies valid.", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
))]
pub async fn get_check() -> impl IntoResponse {
    GenericRsp::create("Cookies valid.")
}
