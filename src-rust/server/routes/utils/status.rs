use crate::constants::version::get_version;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct StatusResponseBody {
    /// Current local server time.
    pub server_time: DateTime<Local>,
    /// Current yomuyume version.
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Your test string.
    pub echo: Option<String>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct StatusRequest {
    ///  A test string to test your request body.
    pub echo: Option<String>,
}

#[utoipa::path(get, path = "/api/utils/status", params(StatusRequest), responses(
    (status = 200, description = "Status check successful", body = StatusResponseBody)
))]
pub async fn get_status(query: Query<StatusRequest>) -> impl IntoResponse {
    let echo = query.echo.clone();
    let version = get_version();
    (
        StatusCode::OK,
        Json(StatusResponseBody {
            server_time: Local::now(),
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
            server_time: Local::now(),
            version,
            echo,
        }),
    )
}
