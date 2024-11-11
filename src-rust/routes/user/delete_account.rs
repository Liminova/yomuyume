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
    routes::{check_pass, Mailer},
    types::{temp_code_purpose::TempCodePurpose, UserID},
    AppError, AppState,
};

/// user delete
///
/// send an email to the user with a code to confirm the deletion
#[utoipa::path(get, path = "/api/user/delete", responses(
    (status = 200, description = "code sent to user's email"),
    (status = 401, description = "unauthorized", body = String),
    (status = 429, description = "too many requests"),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_delete_account(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
) -> Result<Response, AppError> {
    let mailer = Mailer::from(&app_state.config)?;

    // too many request
    let temp_code_record = sqlx::query!(
        r#"SELECT created_at
        FROM temp_codes
        WHERE purpose = $1 AND user_id = $2"#,
        TempCodePurpose::DeleteAccount as TempCodePurpose,
        user_id
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
    let new_code = app_state.id_generator.secure();
    let code = sqlx::query!(
        r#"INSERT INTO temp_codes (purpose, user_id, code, created_at)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (purpose, user_id)
        DO UPDATE SET created_at = $4
        RETURNING code"#,
        TempCodePurpose::DeleteAccount as TempCodePurpose,
        user_id,
        new_code.as_str(),
        Utc::now()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't insert temp code")?
    .code;

    // in4 for email
    let (username, user_email) = sqlx::query!(
        r#"SELECT username, email FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't find user")
    .map(|record| (record.username, record.email))?;

    mailer.send(
        &username,
        &user_email,
        format!("{} - Delete your password", &app_state.config.app_name),
        format!(
            "Hello, {}!\n\n\
            // You have requested to delete your account. Please copy the following code into the app to continue:\n\n\
            {}\n\n\
            If you did not request to delete your account, please ignore this email.\n\n\
            Best regards,\n\
            The {} team",
            &username,
            &code,
            &app_state.config.app_name,
        ),
    ).map(|_| Ok((StatusCode::OK).into_response()))?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct DeleteRequestBody {
    pub code: String,
    pub password: String,
}

/// confirm user delete
///
/// the user provides the code received by email
#[utoipa::path(post, path = "/api/user/delete", responses(
    (status = 200, description = "user deleted"),
    (status = 400, description = "bad request", body = String),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn post_delete_account(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Json(query): Json<DeleteRequestBody>,
) -> Result<Response, AppError> {
    if query.password.is_empty() || query.code.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "password and code cannot be empty").into_response());
    }

    // check password
    let password_hash = sqlx::query!(r#"SELECT password_hash FROM users WHERE id = $1"#, user_id)
        .fetch_one(&app_state.pool)
        .await
        .context("can't find user")?
        .password_hash;
    if !check_pass(&password_hash, &query.password) {
        return Ok((StatusCode::BAD_REQUEST, "invalid password").into_response());
    }

    // check temp code
    let code_creation_time = sqlx::query!(
        r#"DELETE FROM temp_codes WHERE code = $1 AND purpose = $2 AND user_id = $3 RETURNING created_at"#,
        query.code.as_str(),
        TempCodePurpose::DeleteAccount as TempCodePurpose,
        user_id,
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't get temp code creation time")?
    .map(|record| record.created_at);
    match code_creation_time {
        Some(creation_time) => {
            if Utc::now() - creation_time > chrono::Duration::minutes(5) {
                return Ok((StatusCode::BAD_REQUEST, "code expired").into_response());
            }
        }
        None => {
            return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
        }
    };

    sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, user_id)
        .execute(&app_state.pool)
        .await
        .context("can't delete user")?;

    Ok((StatusCode::OK).into_response())
}
