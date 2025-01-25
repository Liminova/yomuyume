use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::{check_pass, Mailer},
    traits::chrono_utils::ChronoUtils,
    types::{temp_code_purpose::TempCodePurpose, UserID},
    utils::{
        app_error::AppError,
        app_state::AppState,
        config::{TEMP_CODE_EXPIRED_AFTER, TEMP_CODE_REQUEST_RATE_LIMIT},
    },
};

/// delete account
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
    let now = Utc::now();

    // too many request
    if let Some(created_at) = sqlx::query!(
        "SELECT created_at
        FROM temp_codes
        WHERE purpose = $1 AND user_id = $2",
        TempCodePurpose::DeleteAccount as TempCodePurpose,
        user_id.as_ref(),
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query temp code")?
    .map(|r| r.created_at)
    {
        if created_at.inside(&now, &Duration::seconds(TEMP_CODE_REQUEST_RATE_LIMIT)) {
            return Ok((StatusCode::TOO_MANY_REQUESTS).into_response());
        }
    };

    // get temp code
    let code = sqlx::query!(
        "INSERT INTO temp_codes (purpose, user_id, code, created_at)
        VALUES ($1, $2, $3, $4) ON CONFLICT (purpose, user_id) DO
        UPDATE
        SET created_at = $4
        RETURNING code",
        TempCodePurpose::DeleteAccount as TempCodePurpose,
        user_id.as_ref(),
        app_state.id_generator.secure(),
        now
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't upsert temp code")?
    .code;

    // in4 for email
    let (username, email) = {
        let user = app_state
            .user_cache
            .get(&user_id)
            .context("can't get cached user, this should not happen")?;
        (user.username.clone(), user.email.clone())
    };

    mailer.send(
        &username,
        &email,
        format!("{} - Delete your password", &app_state.config.app_name),
        format!(
            "Hello, {username}!\n\n\
            // You have requested to delete your account. Please copy the following code into the app to continue:\n\n\
            {code}\n\n\
            If you did not request to delete your account, please ignore this email.\n\n\
            Best regards,\n\
            The {} team",
            &app_state.config.app_name,
        ),
    ).map(|_| Ok(StatusCode::OK.into_response()))?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct DeleteRequestBody {
    pub code: String,
    pub password: String,
}

/// confirm delete account
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

    if !check_pass(
        &sqlx::query!(
            "SELECT password_hash, id FROM users WHERE id = $1",
            user_id.as_ref()
        )
        .fetch_one(&app_state.pool)
        .await
        .context("can't query user")?
        .password_hash,
        &query.password,
    ) {
        return Ok((StatusCode::BAD_REQUEST, "invalid password").into_response());
    }

    // check temp code expiration
    if let Some(created_at) = sqlx::query!(
        "DELETE FROM temp_codes
        WHERE code = $1
            AND purpose = $2
            AND user_id = $3
        RETURNING created_at",
        query.code.as_str(),
        TempCodePurpose::DeleteAccount as TempCodePurpose,
        user_id.as_ref(),
    )
    .fetch_optional(&app_state.pool)
    .await
    .context("can't query temp code")?
    .map(|record| record.created_at)
    {
        if created_at.outside(&Utc::now(), &Duration::seconds(TEMP_CODE_EXPIRED_AFTER)) {
            return Ok((StatusCode::BAD_REQUEST, "code expired").into_response());
        }
    } else {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    };

    // nuke
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id.as_ref())
        .execute(&app_state.pool)
        .await
        .context("can't delete user")?;
    app_state.session_cache.retain(|_, v| *v != user_id);
    app_state.user_cache.remove(&user_id);

    Ok(StatusCode::OK.into_response())
}
