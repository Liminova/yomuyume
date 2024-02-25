use crate::constants::version::get_version;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use chrono::Local;

pub use bridge::routes::utils::{StatusRequest, StatusResponseBody};

#[utoipa::path(get, path = "/api/utils/status", params(StatusRequest), responses(
    (status = 200, description = "Status check successful", body = StatusResponseBody)
))]
pub async fn get_status(query: Query<StatusRequest>) -> impl IntoResponse {
    let echo = query.echo.clone();
    let version = get_version();
    (
        StatusCode::OK,
        Json(StatusResponseBody {
            server_time: Local::now().to_string(),
            version,
            echo,
        }),
    )
}

#[utoipa::path(post, path = "/api/utils/status", responses(
    (status = 200, description = "Status check successful", body = StatusResponseBody)
))]
pub async fn post_status(query: Option<Json<StatusRequest>>) -> impl IntoResponse {
    let echo = query.and_then(|q| q.echo.clone());
    let version = get_version();
    (
        StatusCode::OK,
        Json(StatusResponseBody {
            server_time: Local::now().to_string(),
            version,
            echo,
        }),
    )
}
