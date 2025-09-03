use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

use crate::{
    AppState, database,
    routes::{check_pass, errors::InternalErr},
    utils::{
        constants::{CookieName, LOGIN_PATH},
        nanoid::nanoid,
        result_utils::ResultUtils,
    },
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login
#[utoipa::path(
    post,
    path = LOGIN_PATH,
    responses(
        (status = 200, description = "Login successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String),
    ))
]
pub async fn post_login(
    State(app_state): State<Arc<AppState>>,
    query: Json<LoginRequest>,
) -> Result<Response, InternalErr> {
    let read_txn = app_state
        .db
        .user
        .begin_read()
        .log_err(|e| error!("can't begin read transaction: {e}"))?;

    let Some(user_id) = read_txn
        .open_table(database::user::USER_EMAIL_TO_ID)
        .log_err(|e| error!("can't open users table: {e}"))?
        .get(&query.email)
        .log_err(|e| error!("can't query users table: {e}"))?
        .map(|r| r.value())
    else {
        return Ok((StatusCode::BAD_REQUEST, "invalid email or password").into_response());
    };

    if read_txn
        .open_table(database::user::USERS)
        .log_err(|e| error!("can't open users table: {e}"))?
        .get(&user_id)
        .log_err(|e| error!("can't query users table: {e}"))?
        .map(|r| r.value().password_hash)
        .filter(|password_hash| check_pass(password_hash, &query.password))
        .is_none()
    {
        return Ok((StatusCode::BAD_REQUEST, "invalid email or password").into_response());
    };

    let session_secret = nanoid();

    let write_txn = app_state
        .db
        .user
        .begin_write()
        .log_err(|e| error!("can't begin write transaction: {e}"))?;

    write_txn
        .open_multimap_table(database::user::SESSIONS)
        .log_err(|e| error!("can't open sessions table: {e}"))?
        .insert(
            &user_id,
            &database::user::Session {
                // TODO: detect device name/type
                device: None,
                session_secret: session_secret.clone(),
            },
        )
        .log_err(|e| error!("can't insert session entry: {e}"))?;

    write_txn
        .commit()
        .log_err(|e| error!("can't commit write transaction: {e}"))?;

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (
                header::SET_COOKIE,
                Cookie::build((CookieName::UserID.as_ref(), user_id))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .to_string(),
            ),
            (
                header::SET_COOKIE,
                Cookie::build((CookieName::SessionSecret.as_ref(), session_secret))
                    .path("/")
                    .secure(true)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .to_string(),
            ),
        ]),
    )
        .into_response())
}
