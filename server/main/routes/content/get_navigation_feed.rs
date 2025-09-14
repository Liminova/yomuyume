#![allow(clippy::unused_async, unused_variables)]

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, routes::errors::InternalError, utils::constants::GET_NAVIGATION_FEED_PATH};

#[derive(Debug, Serialize, Deserialize)]
pub enum SeriesOrCategory {
    #[serde(rename = "series")]
    Series,
    #[serde(rename = "category")]
    Category,
}

/// Get navigation feed
#[utoipa::path(
    get,
    path = GET_NAVIGATION_FEED_PATH,
    responses(
        (status = 200, description = "Get navigation feed success", body = String),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    params(
        ("series_or_category" = Option<String>, Path, description = "Either 'category' or 'series'"),
        ("id" = Option<String>, Path, description = "Category ID or Series ID"),
    )
)]
pub async fn get_navigation_feed(
    State(app_state): State<Arc<AppState>>,
    Path((series_or_category, id)): Path<(SeriesOrCategory, String)>,
) -> Result<Response, InternalError> {
    todo!()
}
