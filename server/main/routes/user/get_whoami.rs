use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::error;
use utoipa::ToSchema;

use crate::{
    AppState,
    routes::{UserIDExtension, errors::InternalError},
    utils::{constants::GET_WHOAMI_PATH, result_utils::ResultUtils},
};

#[skip_serializing_none]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct WhoAmIResponse {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub created_at: Option<String>,
    pub verified_at: Option<String>,
}

/// Get current logged in user
#[utoipa::path(
    get,
    path = GET_WHOAMI_PATH,
    responses(
        (status = 200, description = "Who am I response", body = WhoAmIResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("user-id" = [], "session-secret" = []))
)]
pub async fn get_whoami(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
) -> Result<Response, InternalError> {
    let Some(user_id) = user_id.0 else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    if let Some(user) = sqlx::query!(
        "SELECT id, username, email, created_at, verified_at FROM users WHERE id = ?",
        user_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .log_err(|e| error!("can't query user info: {e:?}"))?
    {
        return Ok((
            StatusCode::OK,
            Json(WhoAmIResponse {
                user_id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.map(|c| c.to_string()),
                verified_at: user.verified_at.map(|v| v.to_string()),
            }),
        )
            .into_response());
    }

    Ok(StatusCode::NOT_FOUND.into_response())
}
