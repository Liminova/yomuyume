use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::errors::InternalErr,
    structs::ids::UserID,
    utils::{app_state::AppState, constants::USER_MODIFY_PATH},
};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ModifyRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

/// Modify user information
#[utoipa::path(
    post,
    path = USER_MODIFY_PATH,
    responses(
        (status = 200, description = "Modify user successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Unauthorized", body = String),
        (status = 500, description = "Internal server error", body = String)
    ),
    security(("session-id" = [], "session-secret" = [])))
]
pub async fn post_modify(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Json(body): Json<ModifyRequest>,
) -> Result<Response, InternalErr> {
    let username = body.username.unwrap_or_default();
    let email = body.email.unwrap_or_default();
    let now = Utc::now();

    sqlx::query!(
        "UPDATE users
        SET username = CASE
                WHEN $1 = '' THEN username
                ELSE $1
            END,
            email = CASE
                WHEN $2 = '' THEN email
                ELSE $2
            END,
            updated_at = $3
        WHERE id = $4",
        &username,
        &email,
        &now,
        user_id.as_ref()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e}");
        InternalErr::DB(e)
    })?;

    let mut user = app_state.cacher.users.get_mut(&user_id).ok_or_else(|| {
        let e = InternalErr::WriteCache("user".to_string());
        tracing::error!("{e}");
        e
    })?;
    user.value_mut().username = username;
    user.value_mut().email = email;
    user.value_mut().updated_at = Some(now);
    drop(user);

    Ok(StatusCode::OK.into_response())
}
