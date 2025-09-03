use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use crate::{
    routes::errors::InternalErr,
    structs::ids::UserID,
    utils::{app_state::AppState, constants::WHOAMI_PATH},
};

#[skip_serializing_none]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct WhoAmIResponse {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub ip_address: String,
    pub updated_at: Option<String>,
    pub verified_at: Option<String>,
}

/// Get current logged in user
#[utoipa::path(
    get,
    path = WHOAMI_PATH,
    responses(
        (status = 200, description = "Get whoami successful", body = WhoAmIResponse),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
    security(("user-id" = [], "session-secret" = [])))
]
pub async fn get_whoami(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
) -> Result<Response, InternalErr> {
    if let Some(cached_user) = app_state.cacher.users.get(&user_id) {
        return Ok((
            StatusCode::OK,
            Json(WhoAmIResponse {
                user_id: user_id.as_ref(),
                username: cached_user.username.clone(),
                email: cached_user.email.clone(),
                profile_picture: cached_user.profile_picture.clone(),
                ip_address: cached_user.ip_address.clone(),
                updated_at: cached_user.updated_at.map(|d| d.to_rfc3339()),
                verified_at: cached_user.verified_at.map(|v| v.to_string()),
            }),
        )
            .into_response());
    }

    if let Some(user) = sqlx::query!(
        "SELECT id, username, email, profile_picture, ip_address, updated_at, verified_at
         FROM users WHERE id = $1",
        user_id.as_ref()
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })? {
        return Ok((
            StatusCode::OK,
            Json(WhoAmIResponse {
                user_id: user.id,
                username: user.username,
                email: user.email,
                profile_picture: user.profile_picture,
                ip_address: user.ip_address,
                updated_at: user.updated_at.map(|d| d.to_rfc3339()),
                verified_at: user.verified_at.map(|v| v.to_string()),
            }),
        )
            .into_response());
    }

    Ok((StatusCode::NOT_FOUND, "User not found").into_response())
}
