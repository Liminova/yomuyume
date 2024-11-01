use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::Mailer,
    types::{custom_id::CustomID, temp_code_purpose::TempCodePurpose},
    AppError, AppState,
};

/// Send an email to the user with a code to validate their email address.
#[utoipa::path(get, path = "/api/user/verify", responses(
    (status = 200, description = "Verification email sent"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 429, description = "Too many requests", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_validate_email(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<String>,
) -> Result<Response, AppError> {
    let mailer = Mailer::from(&app_state.config)?;

    // check user
    let user_record = sqlx::query!(
        r#"SELECT id, username, email, verified_at FROM users WHERE id = $1"#,
        user_id.as_str()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't find user")?;
    if let Some(ref verified_at) = user_record.verified_at {
        return Ok((
            StatusCode::BAD_REQUEST,
            format!("user is already verified at {}", verified_at.to_rfc3339()),
        )
            .into_response());
    }

    // too many requests
    let temp_code_record = sqlx::query!(
        r#"SELECT created_at
        FROM temp_codes
        WHERE purpose = $1 AND user_id = $2"#,
        TempCodePurpose::ValidateEmail as TempCodePurpose,
        user_record.id.as_str()
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't find temp code")?;
    if let Some(ref record) = temp_code_record {
        if Utc::now() - record.created_at < chrono::Duration::minutes(5) {
            {
                return Ok((StatusCode::TOO_MANY_REQUESTS).into_response());
            }
        }
    }

    // get temp code
    let new_code = CustomID::new();
    let code = sqlx::query!(
        r#"INSERT INTO temp_codes (purpose, user_id, code, created_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (purpose, user_id)
        DO UPDATE SET created_at = $4
        RETURNING code"#,
        TempCodePurpose::ValidateEmail as TempCodePurpose,
        user_record.id.as_str(),
        new_code.as_str(),
        Utc::now()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't insert temp code")?
    .code;

    mailer
        .send(
            &user_record.username,
            &user_record.email,
            format!("{} - Verify your account", &app_state.config.app_name),
            format!(
                "Hello {},\n\n\
            You have requested to verify your account. \
            Please click copy the following token into the app to continue:\n\n\
            {}\n\n\
            If you did not request this, please ignore this email.\n\n\
            Thanks,\n\
            The {} Team",
                &user_record.username, &code, &app_state.config.app_name,
            ),
        )
        .map(|_| Ok((StatusCode::OK).into_response()))?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ValidateEmailRequestBody {
    pub code: String,
}

/// The user provides the code received by email to verify their email address.
#[utoipa::path(post, path = "/api/user/verify", responses(
    (status = 200, description = "Account verification successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn post_validate_email(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<String>,
    Json(query): Json<ValidateEmailRequestBody>,
) -> Result<Response, AppError> {
    // check user
    let user_record = sqlx::query!(
        r#"SELECT id, username, email, verified_at FROM users WHERE id = $1"#,
        user_id.as_str()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't find user")?;
    if let Some(ref verified_at) = user_record.verified_at {
        return Ok((
            StatusCode::BAD_REQUEST,
            format!("user is already verified at {}", verified_at.to_rfc3339()),
        )
            .into_response());
    }

    // check temp code
    let code_creation_time = sqlx::query!(
        r#"DELETE FROM temp_codes
        WHERE code = $1
        AND purpose = $2
        AND user_id = $3
        RETURNING created_at"#,
        query.code.as_str(),
        TempCodePurpose::ValidateEmail as TempCodePurpose,
        user_record.id.as_str()
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't find temp code")?
    .map(|record| record.created_at);
    if let Some(code_creation_time) = code_creation_time {
        if Utc::now() - code_creation_time > chrono::Duration::minutes(5) {
            return Ok((StatusCode::BAD_REQUEST, "code expired").into_response());
        }
    } else {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    }

    sqlx::query!(
        r#"UPDATE users SET verified_at = $1 WHERE id = $2"#,
        Utc::now(),
        user_record.id.as_str()
    )
    .execute(&app_state.pool)
    .await
    .context("can't update user")?;

    Ok((StatusCode::OK).into_response())
}
