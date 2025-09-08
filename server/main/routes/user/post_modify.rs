use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    AppState,
    routes::{UserIDExtension, errors::InternalError},
    utils::{constants::POST_USER_MODIFY_PATH, result_utils::ResultUtils},
};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ModifyRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

/// Modify user information
#[utoipa::path(
    post,
    path = POST_USER_MODIFY_PATH,
    responses(
        (status = 200, description = "Modify user successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(("user-id" = [], "session-secret" = []))
)]
pub async fn post_modify(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserIDExtension>,
    Json(body): Json<ModifyRequest>,
) -> Result<Response, InternalError> {
    let Some(user_id) = user_id.0 else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };
    let username = body.username.unwrap_or_default();
    let email = body.email.unwrap_or_default();

    sqlx::query!(
        "UPDATE users SET
            username = CASE
                WHEN :username = '' THEN username
                ELSE :username
            END,
            email = CASE
                WHEN :email = '' THEN email
                ELSE :email
            END
        WHERE id = :id",
        username,
        email,
        user_id
    )
    .execute(&app_state.pool)
    .await
    .log_err(|e| error!("can't upsert user info: {e}"))?;

    Ok(StatusCode::OK.into_response())
}
