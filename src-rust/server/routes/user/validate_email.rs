use std::sync::Arc;

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
    traits::chrono_utils::ChronoUtils,
    types::{temp_code_purpose::TempCodePurpose, UserID},
    utils::{
        app_error::AppError,
        app_state::AppState,
        config::{TEMP_CODE_EXPIRED_AFTER, TEMP_CODE_REQUEST_RATE_LIMIT},
    },
};

/// validate email
///
/// send an email to the userwith a code to validate their email address
#[utoipa::path(get, path = "/api/user/verify", responses(
    (status = 200, description = "verification email sent"),
    (status = 400, description = "bad request", body = String),
    (status = 401, description = "unauthorized", body = String),
    (status = 429, description = "too many requests"),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn get_validate_email(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
) -> Result<Response, AppError> {
    let mailer = Mailer::from(&app_state.config).map_err(|e| {
        tracing::error!("{e:?}");
        AppError::Mailer(e)
    })?;

    let (username, email) = {
        let user = app_state.user_cache.get(&user_id).ok_or_else(|| {
            let e = AppError::ReadCache("user".to_string());
            tracing::error!("{e:?}");
            e
        })?;

        if let Some(verified_at) = user.verified_at {
            return Ok((
                StatusCode::BAD_REQUEST,
                format!("user is already verified at {}", verified_at.to_rfc3339()),
            )
                .into_response());
        };

        (user.username.clone(), user.email.clone())
    };

    // too many requests
    if let Some(created_at) = sqlx::query!(
        "SELECT created_at
        FROM temp_codes
        WHERE purpose = $1
            AND user_id = $2",
        TempCodePurpose::ValidateEmail as TempCodePurpose,
        user_id.as_ref(),
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        AppError::DB(e)
    })?
    .map(|record| record.created_at)
    {
        if created_at.inside(
            &Utc::now(),
            &chrono::Duration::seconds(TEMP_CODE_REQUEST_RATE_LIMIT),
        ) {
            return Ok((StatusCode::TOO_MANY_REQUESTS).into_response());
        };
    }

    // get temp code
    let code = sqlx::query!(
        "INSERT INTO temp_codes (purpose, user_id, code, created_at)
        VALUES ($1, $2, $3, $4) ON CONFLICT (purpose, user_id) DO
        UPDATE
        SET created_at = $4
        RETURNING code",
        TempCodePurpose::ValidateEmail as TempCodePurpose,
        user_id.as_ref(),
        app_state.id_generator.secure(),
        Utc::now()
    )
    .fetch_one(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        AppError::DB(e)
    })?
    .code;

    mailer
        .send(
            &username,
            &email,
            format!("{} - Verify your account", &app_state.config.app_name),
            format!(
                "Hello {username},\n\n\
                You have requested to verify your account.\
                Please click copy the following token into the app to continue:\n\n\
                {code}\n\n\
                If you did not request this, please ignore this email.\n\n\
                Thanks,\n\
                The {} Team",
                &app_state.config.app_name,
            ),
        )
        .map(|_| Ok(StatusCode::OK.into_response()))
        .map_err(|e| {
            tracing::error!("{e:?}");
            AppError::Mailer(e)
        })?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ValidateEmailRequestBody {
    pub code: String,
}

/// confirm validate email
///
/// the user provides the code received by email to verify their email address
#[utoipa::path(post, path = "/api/user/verify", responses(
    (status = 200, description = "account verification successful"),
    (status = 400, description = "bad request", body = String),
    (status = 401, description = "unauthorized", body = String),
    (status = 500, description = "internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn post_validate_email(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Json(query): Json<ValidateEmailRequestBody>,
) -> Result<Response, AppError> {
    let user = app_state.user_cache.get(&user_id).ok_or_else(|| {
        let e = AppError::ReadCache("user".to_string());
        tracing::error!("{e:?}");
        e
    })?;
    if let Some(verified_at) = user.verified_at {
        return Ok((
            StatusCode::BAD_REQUEST,
            format!("user is already verified at {}", verified_at.to_rfc3339()),
        )
            .into_response());
    };
    drop(user);

    let now = Utc::now();

    // check temp code expiration
    if let Some(created_at) = sqlx::query!(
        "DELETE FROM temp_codes
        WHERE code = $1
            AND purpose = $2
            AND user_id = $3
        RETURNING created_at",
        query.code.as_str(),
        TempCodePurpose::ValidateEmail as TempCodePurpose,
        user_id.as_ref(),
    )
    .fetch_optional(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        AppError::DB(e)
    })?
    .map(|record| record.created_at)
    {
        if created_at.outside(&now, &chrono::Duration::seconds(TEMP_CODE_EXPIRED_AFTER)) {
            return Ok((StatusCode::BAD_REQUEST, "code expired").into_response());
        }
    } else {
        return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response());
    };

    sqlx::query!(
        "UPDATE users
        SET verified_at = $1
        WHERE id = $2",
        &now,
        user_id.as_ref()
    )
    .execute(&app_state.pool)
    .await
    .map_err(|e| {
        tracing::error!("{e:?}");
        AppError::DB(e)
    })?;
    app_state
        .user_cache
        .get_mut(&user_id)
        .ok_or_else(|| {
            let e = AppError::WriteCache("user".to_string());
            tracing::error!("{e:?}");
            e
        })?
        .value_mut()
        .verified_at = Some(now);

    Ok(StatusCode::OK.into_response())
}
