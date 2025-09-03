use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    AppState, database,
    routes::{
        errors::{InternalErr, RequestErr},
        hash_pass, is_strong,
    },
    utils::{constants::REGISTER_PATH, nanoid::nanoid, result_utils::ResultUtils},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: Option<String>,
    pub email: String,
    pub password: String,
}

/// Register
#[utoipa::path(
    post,
    path = REGISTER_PATH,
    responses(
        (status = 200, description = "Registration successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 409, description = "A conflict has occurred", body = String),
        (status = 500, description = "Internal server error", body = String),
    ))
]
pub async fn post_register(
    State(app_state): State<Arc<AppState>>,
    query: Json<RegisterRequest>,
) -> Result<Response, InternalErr> {
    let Ok(email) = query.email.parse::<lettre::Address>() else {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidEmail).into_response());
    };

    if app_state
        .db
        .user
        .begin_read()
        .log_err(|e| error!("can't begin read transaction: {}", e))?
        .open_table(database::user::USER_EMAIL_TO_ID)
        .log_err(|e| error!("can't open users table: {}", e))?
        .get(&email.to_string())
        .log_err(|e| error!("can't get user by email: {}", e))?
        .is_some()
    {
        return Ok((StatusCode::CONFLICT, RequestErr::EmailAlreadyUsed).into_response());
    }

    if !is_strong(&query.password) {
        return Ok((StatusCode::BAD_REQUEST, RequestErr::WeakPassword).into_response());
    }

    let password_hash = hash_pass(&query.password).map_err(|e| {
        error!("can't hash password: {}", e);
        InternalErr::PasswordHash(e)
    })?;

    let write_txn = app_state
        .db
        .user
        .begin_write()
        .log_err(|e| error!("can't begin write transaction: {}", e))?;

    let user_id = nanoid();

    write_txn
        .open_table(database::user::USERS)
        .log_err(|e| error!("can't open users table: {}", e))?
        .insert(
            user_id.clone(),
            database::user::UserInfo {
                name: None,
                email: email.to_string(),
                password_hash,
                profile_picture: None,
                verified_at: None,
            },
        )
        .log_err(|e| error!("can't insert new user: {}", e))?;

    write_txn
        .open_table(database::user::USER_EMAIL_TO_ID)
        .log_err(|e| error!("can't open user email to id table: {}", e))?
        .insert(email.to_string(), user_id)
        .log_err(|e| error!("can't insert new user email to id mapping: {}", e))?;

    write_txn
        .commit()
        .log_err(|e| error!("can't commit write transaction: {}", e))?;

    Ok(StatusCode::OK.into_response())
}
