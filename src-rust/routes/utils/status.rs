use std::sync::Arc;

use crate::AppState;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, IntoParams, TS)]
#[ts(export)]
pub struct StatusRequestBody {
    ///  A test string to test your request body.
    pub echo: Option<String>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatusResponseBody {
    /// Current local server time.
    pub server_time: String,
    /// Current yomuyume version.
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Your test string.
    pub echo: Option<String>,
}

#[utoipa::path(get, path = "/api/utils/status", responses(
    (status = 200, description = "Status check successful", body = StatusResponseBody)
))]
pub async fn get_status(
    State(app_state): State<Arc<AppState>>,
    query: Query<StatusRequestBody>,
) -> Response {
    let echo = query.echo.clone();
    let version = app_state.config.get_version();

    (
        StatusCode::OK,
        Json(StatusResponseBody {
            server_time: Local::now().to_string(),
            version,
            echo,
        }),
    )
        .into_response()
}

#[utoipa::path(post, path = "/api/utils/status", responses(
    (status = 200, description = "Status check successful", body = StatusResponseBody)
))]
pub async fn post_status(
    State(app_state): State<Arc<AppState>>,
    query: Option<Json<StatusRequestBody>>,
) -> Response {
    let echo = query.and_then(|q| q.echo.clone());
    let version = app_state.config.get_version();
    (
        StatusCode::OK,
        Json(StatusResponseBody {
            server_time: Local::now().to_string(),
            version,
            echo,
        }),
    )
        .into_response()
}
