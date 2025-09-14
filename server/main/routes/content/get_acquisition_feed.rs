#![allow(clippy::unused_async, unused_variables)]

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Response,
};

use crate::{AppState, routes::errors::InternalError, utils::constants::GET_ACQUISITION_FEED_PATH};

/// Get navigation feed
#[allow(unused)]
#[utoipa::path(
    get,
    path = GET_ACQUISITION_FEED_PATH,
    responses(
        (status = 200, description = "Get acquisition feed success", body = String),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    params(
        ("title_id" = String, Path, description = "Category ID or Series ID"),
    )
)]
pub async fn get_acquisition_feed(
    State(app_state): State<Arc<AppState>>,
    Path(title_id): Path<String>,
) -> Result<Response, InternalError> {
    todo!()
}
