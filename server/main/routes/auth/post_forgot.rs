use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{TimeZone, Utc};
use lettre::Address;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{
    AppState,
    routes::{errors::InternalError, hash_pass, user::Mailer},
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
) -> Result<Response, InternalError> {
    let Ok(email) = query.email.parse::<lettre::Address>() else {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
    };
    let email_str = email.to_string();

    let Some(user_id) = sqlx::query!("SELECT id FROM users WHERE email = ?", email_str)
        .fetch_optional(&app_state.pool)
        .await
        .inspect_err(|e| error!("can't query user by email: {e}"))?
        .map(|r| r.id)
    else {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
    };

    match (&query.code, &query.new_password) {
        (Some(code), Some(new_password)) => reset(&app_state, &user_id, code, new_password).await,
        (None, None) => request(&app_state, &user_id, email).await,
        _ => Ok((StatusCode::BAD_REQUEST, "invalid request").into_response()),
    }
}

async fn request(
    app_state: &Arc<AppState>,
    user_id: &str,
    email: Address,
) -> Result<Response, InternalError> {
    let mailer = app_state.config.smtp.as_ref().map(Mailer::from);
    let now = Utc::now();

    if let Some(existing_request) = sqlx::query!(
        "SELECT created_at FROM forgot_password_requests WHERE user_id = ?",
        user_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't query existing forgot password request: {e}"))?
        && Utc
            .from_utc_datetime(&existing_request.created_at)
            .inside(&now, &ForgotPasswordLimit::Cooldown.into())
    {
        #[cfg(debug_assertions)]
        tracing::debug!(
            "forgot password request cooldown for user {} | now {} | prev {}",
            email,
            now,
            existing_request.created_at
        );
        return Ok(StatusCode::TOO_MANY_REQUESTS.into_response());
    };

    let code = nanoid();

    let now_to_insert = now.naive_utc();
    sqlx::query!(
        "INSERT OR REPLACE INTO forgot_password_requests (code, user_id, created_at) VALUES (?, ?, ?)",
        code,
        user_id,
        now_to_insert
    )
    .execute(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't insert forgot password request: {e}"))?;

    let username = sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
        .fetch_optional(&app_state.pool)
        .await
        .inspect_err(|e| error!("can't query username by id: {e}"))?
        .map(|r| r.username);

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
        ).inspect_err(|e| error!("can't send forgot password email: {e}"))?;
    } else {
        info!(
            "forgot password code for user {}: {code}",
            email.to_string()
        );
    }

    Ok(StatusCode::OK.into_response())
}

async fn reset(
    app_state: &Arc<AppState>,
    user_id: &str,
    code: &str,
    new_password: &str,
) -> Result<Response, InternalError> {
    let Some(existing_request) = sqlx::query!(
        "SELECT code, created_at FROM forgot_password_requests WHERE user_id = ?",
        user_id
    )
    .fetch_optional(&app_state.pool)
    .await
    .inspect_err(|e| error!("can't query existing forgot password request: {e}"))?
    else {
        return Ok((StatusCode::BAD_REQUEST, "no forgot password request found").into_response());
    };

    if code != existing_request.code
        || Utc
            .from_utc_datetime(&existing_request.created_at)
            .outside(&Utc::now(), &ForgotPasswordLimit::ExpiredAfter.into())
    {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    }

    let mut tx = app_state
        .pool
        .begin()
        .await
        .inspect_err(|e| error!("can't begin SQL transaction: {e}"))?;

    sqlx::query!(
        "DELETE FROM forgot_password_requests WHERE user_id = ?",
        user_id
    )
    .execute(&mut *tx)
    .await
    .inspect_err(|e| error!("can't delete used forgot password request: {e}"))?;

    let new_password_hash = hash_pass(new_password).map_err(|e| {
        error!("can't hash new password: {e}");
        InternalError::PasswordHash(e)
    })?;

    sqlx::query!(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        new_password_hash,
        user_id
    )
    .execute(&mut *tx)
    .await
    .inspect_err(|e| error!("can't update user password: {e}"))?;

    tx.commit()
        .await
        .inspect_err(|e| error!("can't commit transaction: {e}"))?;

    Ok(StatusCode::OK.into_response())
}
