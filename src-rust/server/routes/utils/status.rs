use crate::{constants::version::get_version, MyResponseBuilder};
use axum::{extract::Query, http::HeaderMap, response::IntoResponse, Json};
use chrono::Local;

pub use bridge::routes::utils::{StatusRequest, StatusResponseBody};

#[utoipa::path(get, path = "/api/utils/status", params(StatusRequest), responses(
    (status = 200, description = "Status check successful", body = StatusResponseBody)
))]
pub async fn get_status(header: HeaderMap, query: Query<StatusRequest>) -> impl IntoResponse {
    let builder = MyResponseBuilder::new(header);

    let echo = query.echo.clone();
    let version = get_version();

    builder.success(StatusResponseBody {
        server_time: Local::now().to_string(),
        version,
        echo,
    })
}

#[utoipa::path(post, path = "/api/utils/status", responses(
    (status = 200, description = "Status check successful", body = StatusResponseBody)
))]
pub async fn post_status(
    header: HeaderMap,
    query: Option<Json<StatusRequest>>,
) -> impl IntoResponse {
    let builder = MyResponseBuilder::new(header);

    let echo = query.and_then(|q| q.echo.clone());
    let version = get_version();
    builder.success(StatusResponseBody {
        server_time: Local::now().to_string(),
        version,
        echo,
    })
}
