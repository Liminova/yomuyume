use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use lettre::Address;
use redb::{ReadTransaction, ReadableDatabase};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{
    AppState,
    database::{self, user::UserID},
    routes::{errors::InternalErr, hash_pass, user::Mailer},
    utils::{
        chrono_utils::ChronoUtils,
        constants::{FORGOT_PATH, ForgotPasswordLimit},
        nanoid::nanoid,
        result_utils::ResultUtils,
    },
};

#[derive(Debug, ToSchema, Deserialize, Serialize)]
pub struct ForgotRequest {
    pub email: String,
    pub code: Option<String>,
    pub new_password: Option<String>,
}

/// Forgot password
///
/// Send the first with only the email to request the code,
/// send the second one with all the fields to reset the password.
#[utoipa::path(
    post,
    path = FORGOT_PATH,
    responses(
        (status = 200, description = "Request successful"),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String),
    ))
]
pub async fn post_forgot(
    State(app_state): State<Arc<AppState>>,
    query: Json<ForgotRequest>,
) -> Result<Response, InternalErr> {
    let read_txn = app_state
        .db
        .user
        .begin_read()
        .log_err(|e| error!("can't begin read transaction: {e}"))?;

    let Ok(email) = query.email.parse::<lettre::Address>() else {
        return Ok((StatusCode::BAD_REQUEST, "Invalid email").into_response());
    };

    let Some(user_id) = read_txn
        .open_table(database::user::USER_EMAIL_TO_ID)
        .log_err(|e| error!("can't open users table: {e}"))?
        .get(&query.email)
        .log_err(|e| error!("can't get user by email: {e}"))?
        .map(|r| r.value())
    else {
        return Ok((StatusCode::BAD_REQUEST, "Invalid email").into_response());
    };

    match (&query.code, &query.new_password) {
        (Some(code), Some(new_password)) => {
            reset(&app_state, read_txn, user_id, code, new_password).await
        }
        (None, None) => request(&app_state, read_txn, user_id, email).await,
        _ => return Ok((StatusCode::BAD_REQUEST, "Invalid request").into_response()),
    }
}

async fn request(
    app_state: &Arc<AppState>,
    read_txn: ReadTransaction,
    user_id: UserID,
    email: Address,
) -> Result<Response, InternalErr> {
    let mailer = app_state.config.smtp.as_ref().map(Mailer::from);
    let now = Utc::now();

    if read_txn
        .open_table(database::user::FORGOT_PASSWORD)
        .log_err(|e| error!("can't open forgot password table: {e}"))?
        .get(&user_id)
        .log_err(|e| error!("can't get forgot password requests: {e}"))?
        .filter(|r| {
            r.value()
                .created_at
                .inside(&now, &ForgotPasswordLimit::Cooldown.into())
        })
        .is_some()
    {
        return Ok(StatusCode::TOO_MANY_REQUESTS.into_response());
    }

    let username = read_txn
        .open_table(database::user::USERS)
        .log_err(|e| error!("can't open users table: {e}"))?
        .get(&user_id)
        .log_err(|e| error!("can't get user info: {e}"))?
        .and_then(|r| r.value().name);

    let code = nanoid();

    let write_txn = app_state
        .db
        .user
        .begin_write()
        .log_err(|e| error!("can't begin write transaction: {e}"))?;

    write_txn
        .open_table(database::user::FORGOT_PASSWORD)
        .log_err(|e| error!("can't open forgot password table: {e}"))?
        .insert(
            &user_id,
            database::user::ForgotPassword {
                created_at: now,
                code: code.clone(),
            },
        )
        .log_err(|e| error!("can't insert forgot password request: {e}"))?;

    write_txn
        .commit()
        .log_err(|e| error!("can't commit transaction: {e}"))?;

    if let Some(mailer) =
        mailer.and_then(|mailer| mailer.okay(|e| error!("can't create mailer: {e}")))
    {
        mailer.send(
            username,
            email,
            "Yomuyume - reset password",
            format!(
                "You have requested to reset your password. Please copy the following code into the app to continue:\n\n\
                {code}\n\n\
                If you don't recognize this action or don't own this account, ignore this email.",
            ),
        ).log_err(|e| error!("can't send forgot password email: {e}"))?;
    } else {
        info!(
            "Forgot password code for user {}: {code}",
            email.to_string()
        );
    }

    Ok(StatusCode::OK.into_response())
}

async fn reset(
    app_state: &Arc<AppState>,
    read_txn: ReadTransaction,
    user_id: UserID,
    code: &str,
    new_password: &str,
) -> Result<Response, InternalErr> {
    let Some(forgot_password) = read_txn
        .open_table(database::user::FORGOT_PASSWORD)
        .log_err(|e| error!("can't open forgot password table: {e}"))?
        .get(&user_id)
        .log_err(|e| error!("can't get forgot password requests: {e}"))?
        .map(|r| r.value())
    else {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    };

    if code != forgot_password.code {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    }

    if forgot_password
        .created_at
        .outside(&Utc::now(), &ForgotPasswordLimit::ExpiredAfter.into())
    {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    }

    let new_password_hash = hash_pass(new_password).map_err(|e| {
        error!("can't hash new password: {e}");
        InternalErr::PasswordHash(e)
    })?;

    let write_txn = app_state
        .db
        .user
        .begin_write()
        .log_err(|e| error!("can't begin write transaction: {e}"))?;

    write_txn
        .open_table(database::user::FORGOT_PASSWORD)
        .log_err(|e| error!("can't open forgot password table: {e}"))?
        .remove(&user_id)
        .log_err(|e| error!("can't remove forgot password request: {e}"))?;

    write_txn
        .open_table(database::user::USERS)
        .log_err(|e| error!("can't open users table: {e}"))?
        .get_mut(&user_id)
        .log_err(|e| error!("can't get user info: {e}"))?
        .map(|r| {
            r.value().password_hash = new_password_hash;
        });

    write_txn
        .commit()
        .log_err(|e| error!("can't commit transaction: {e}"))?;

    Ok(StatusCode::OK.into_response())
}
