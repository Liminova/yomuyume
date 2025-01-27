use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use crate::{
    types::UserID,
    utils::{app_error::AppError, app_state::AppState},
};

#[skip_serializing_none]
#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct WhoAmIResponseBody {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub profile_picture: Option<String>,
    pub ip_address: String,
    pub updated_at: Option<String>,
    pub verified_at: Option<String>,
}

/// whoami
///
/// get the current user's information
#[utoipa::path(get, path = "/api/user/whoami", responses(
    (status = 200, description = "get whoami successful", body = WhoAmIResponseBody),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_whoami(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
) -> Result<Response, AppError> {
    let user = app_state.user_cache.get(&user_id).ok_or_else(|| {
        let e = AppError::ReadCache("user".to_string());
        tracing::error!("{e:?}");
        e
    })?;

    Ok((
        StatusCode::OK,
        Json(WhoAmIResponseBody {
            user_id: user_id.as_ref(),
            username: user.username.clone(),
            email: user.email.clone(),
            profile_picture: user.profile_picture.clone(),
            ip_address: user.ip_address.clone(),
            updated_at: user.updated_at.map(|d| d.to_rfc3339()),
            verified_at: user.verified_at.map(|v| v.to_string()),
        }),
    )
        .into_response())
}
