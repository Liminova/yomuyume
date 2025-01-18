use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::{hash_pass, Mailer},
    types::temp_code_purpose::TempCodePurpose,
    utils::{app_error::AppError, app_state::AppState},
};

/// reset password
///
/// send an email to the user with a code to reset the password
#[utoipa::path(get, path = "/api/user/reset", responses(
    (status = 200, description = "code sent to user's email"),
    (status = 400, description = "bad request", body = String),
    (status = 429, description = "too many requests"),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_reset_password(
    State(app_state): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<Response, AppError> {
    if !EmailAddress::is_valid(&email) {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
    }
    let mailer = Mailer::from(&app_state.config)?;

    // valid user
    let user_record = sqlx::query!(
        "SELECT id, username, email, verified_at FROM users WHERE email = $1",
        email.to_string().to_ascii_lowercase()
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query user")?;

    let Some(user_record) = user_record else {
        return Ok((StatusCode::BAD_REQUEST, "user not found").into_response());
    };
    if user_record.verified_at.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "user is not verified").into_response());
    };

    // too many requests
    let temp_code_record = sqlx::query!(
        "SELECT created_at FROM temp_codes WHERE purpose = $1 AND user_id = $2",
        TempCodePurpose::ResetPassword as TempCodePurpose,
        user_record.id
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query temp code")?;
    if let Some(ref record) = temp_code_record {
        if Utc::now() - record.created_at < chrono::Duration::minutes(5) {
            {
                return Ok((StatusCode::TOO_MANY_REQUESTS).into_response());
            }
        }
    }

    // get temp code
    let new_code = app_state.id_generator.secure();
    let code = sqlx::query!(
        "INSERT INTO temp_codes (purpose, user_id, code, created_at)
            VALUES ($1, $2, $3, $4)
        ON CONFLICT (purpose, user_id)
            DO UPDATE SET created_at = $4
        RETURNING code",
        TempCodePurpose::ResetPassword as TempCodePurpose,
        user_record.id,
        new_code.as_str(),
        Utc::now()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't upsert temp code")?
    .code;

    mailer
        .send(
            &user_record.username,
            &user_record.email,
            format!("{} - Reset your password", &app_state.config.app_name),
            format!(
                "Hello, {}!\n\n\
                You have requested to reset your password. Please copy the following code into the app to continue:\n\n\
                {}\n\n\
                If you did not request to reset your password, please ignore this email.\n\n\
                Best regards,\n\
                The {} team",
                &user_record.username,
                &code,
                &app_state.config.app_name,
            ),
        )
        .map(|_| Ok((StatusCode::OK).into_response()))?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ResetRequestBody {
    pub code: String,
    pub new_password: String,
}

/// confirm reset password
///
/// the user provides the code received by email to confirm the password change
#[utoipa::path(post, path = "/api/user/reset", responses(
    (status = 200, description = "password reset successful"),
    (status = 500, description = "internal server error", body = String),
    (status = 400, description = "bad request", body = String),
    (status = 401, description = "unauthorized", body = String),
))]
pub async fn post_reset_password(
    State(app_state): State<Arc<AppState>>,
    Json(query): Json<ResetRequestBody>,
) -> Result<Response, AppError> {
    if query.new_password.is_empty() || query.code.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "password and code cannot be empty").into_response());
    }

    // check temp code
    let temp_code_record = sqlx::query!(
        "DELETE FROM temp_codes WHERE code = $1 AND purpose = $2 RETURNING created_at, user_id",
        query.code.as_str(),
        TempCodePurpose::ResetPassword as TempCodePurpose,
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query temp code")?;
    let temp_code_record = if let Some(record) = temp_code_record {
        if Utc::now() - record.created_at > chrono::Duration::minutes(5) {
            return Ok((StatusCode::BAD_REQUEST, "code expired").into_response());
        }
        record
    } else {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    };

    // update password
    let password_hash = hash_pass(query.new_password)?;
    sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3",
        password_hash.as_str(),
        Utc::now(),
        temp_code_record.user_id
    )
    .execute(&app_state.pool)
    .await
    .context("can't update user")?;

    Ok((StatusCode::OK).into_response())
}
