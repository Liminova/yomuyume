use crate::routes::MyResponseBuilder;
use axum::{http::HeaderMap, response::IntoResponse};

/// Check if the token in the request header/cookie is valid.
#[utoipa::path(get, path = "/api/user/check", responses(
    (status = 200, description = "Cookies valid.", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
))]
pub async fn get_check(header: HeaderMap) -> impl IntoResponse {
    let builder = MyResponseBuilder::new(header);

    builder.generic_success("Cookies valid.")
}
