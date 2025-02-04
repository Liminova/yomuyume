use std::sync::Arc;

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
    routes::{
        check_pass,
        errors::{InternalErr, RequestErr},
        hash_pass, is_strong,
        user::Mailer,
    },
    structs::{id::UserID, temp_code_purpose::TempCodePurpose},
    traits::chrono_utils::ChronoUtils,
    utils::{
        app_state::AppState,
        config::{TEMP_CODE_EXPIRED_AFTER, TEMP_CODE_REQUEST_RATE_LIMIT},
        macros::bail_if_empty,
    },
};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveRequestMode {
    Ask,
    Confirm,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensitiveRequestPurpose {
    DeleteAccount,
    ChangePassword,
    VerifyEmail,
}

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize)]
pub struct SensitiveRequest {
    password: String,
    mode: SensitiveRequestMode,
    purpose: SensitiveRequestPurpose,
    code: Option<String>,
    /// At the moment this is only used for the change
    /// password purpose to set the new password
    payload: Option<String>,
}

/// Perform sensitive action
#[utoipa::path(get, path = "/api/user/sensitive", responses(
    (status = 200, description = "Code sent to user's email"),
    (status = 401, description = "Unauthorized", body = String),
    (status = 429, description = "Too many requests"),
    (status = 500, description = "Internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn post_sensitive(
    State(app_state): State<Arc<AppState>>,
    Extension(user_id): Extension<UserID>,
    Json(req): Json<SensitiveRequest>,
) -> Result<Response, InternalErr> {
    let now = Utc::now();
    let purpose = match &req.purpose {
        SensitiveRequestPurpose::DeleteAccount => TempCodePurpose::DeleteAccount,
        SensitiveRequestPurpose::ChangePassword => TempCodePurpose::ResetPassword,
        SensitiveRequestPurpose::VerifyEmail => TempCodePurpose::ValidateEmail,
    };

    match req.mode {
        SensitiveRequestMode::Ask => {
            // case-specific preludes
            match purpose {
                TempCodePurpose::DeleteAccount | TempCodePurpose::ResetPassword => (),
                TempCodePurpose::ValidateEmail => {
                    if app_state
                        .user_cache
                        .get(&user_id)
                        .map(|u| u.verified_at)
                        .is_some()
                    {
                        return Ok(
                            (StatusCode::BAD_REQUEST, RequestErr::AlreadyVerified).into_response()
                        );
                    }
                }
            };

            let mailer = Mailer::from(&app_state.config).map_err(|e| {
                tracing::error!("{e:?}");
                InternalErr::Mailer(e)
            })?;

            // too many request
            if sqlx::query!(
                "SELECT created_at
                    FROM temp_codes
                    WHERE purpose = $1 AND user_id = $2",
                &purpose as &TempCodePurpose,
                user_id.as_ref(),
            )
            .fetch_optional(&app_state.pool)
            .await
            .map_err(|e| {
                tracing::error!("{e:?}");
                InternalErr::DB(e)
            })?
            .is_some_and(|r| {
                r.created_at
                    .inside(&now, &Duration::seconds(TEMP_CODE_REQUEST_RATE_LIMIT))
            }) {
                return Ok(StatusCode::TOO_MANY_REQUESTS.into_response());
            };

            // get new? code
            let code = sqlx::query!(
                "INSERT INTO temp_codes (purpose, user_id, code, created_at)
                VALUES ($1, $2, $3, $4) ON CONFLICT (purpose, user_id) DO
                UPDATE
                SET created_at = $4
                RETURNING code",
                &purpose as &TempCodePurpose,
                user_id.as_ref(),
                app_state.id_generator.secure(),
                now
            )
            .fetch_one(&app_state.pool)
            .await
            .map_err(|e| {
                tracing::error!("{e:?}");
                InternalErr::DB(e)
            })?
            .code;

            let (username, email) = app_state
                .user_cache
                .get(&user_id)
                .ok_or_else(|| {
                    let e = InternalErr::ReadCache("user".to_string());
                    tracing::error!("{e:?}");
                    e
                })
                .map(|u| (u.username.clone(), u.email.clone()))?;
            let app_name = &app_state.config.app_name;
            let action = match purpose {
                TempCodePurpose::DeleteAccount => "delete your account",
                TempCodePurpose::ResetPassword => "reset your password",
                TempCodePurpose::ValidateEmail => "verify your email",
            };

            mailer.send(
                    &username,
                    &email,
                    format!("{app_name} - Delete your password"),
                    format!(
                        "Hello, {username}!\n\n\
                        // You have requested to {action}. Please copy the following code into the app to continue:\n\n\
                        {code}\n\n\
                        If you don't recognize this action or don't own this account, ignore this email.\n\n\
                        Best regards,\n\
                        The {app_name} team",
                    ),
                )
                .map_err(|e| {
                    tracing::error!("{e:?}");
                    InternalErr::Mailer(e)
                })?;
        }
        SensitiveRequestMode::Confirm => {
            bail_if_empty!(
                req.password,
                Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidCurrentPassword).into_response())
            );
            let Some(code) = req.code.as_ref() else {
                return Ok(
                    (StatusCode::BAD_REQUEST, RequestErr::InvalidPasswordAndCode).into_response(),
                );
            };

            // invalid password
            if !check_pass(
                &sqlx::query!(
                    "SELECT password_hash, id FROM users WHERE id = $1",
                    user_id.as_ref()
                )
                .fetch_one(&app_state.pool)
                .await
                .map_err(|e| {
                    tracing::error!("{e:?}");
                    InternalErr::DB(e)
                })?
                .password_hash,
                &req.password,
            ) {
                return Ok(
                    (StatusCode::BAD_REQUEST, RequestErr::InvalidPasswordAndCode).into_response(),
                );
            }

            // invalid || expired code
            if sqlx::query!(
                "DELETE FROM temp_codes
                WHERE code = $1
                    AND purpose = $2
                    AND user_id = $3
                RETURNING created_at",
                code,
                &purpose as &TempCodePurpose,
                user_id.as_ref(),
            )
            .fetch_optional(&app_state.pool)
            .await
            .map_err(|e| {
                tracing::error!("{e:?}");
                InternalErr::DB(e)
            })?
            .map_or(true, |record| {
                record
                    .created_at
                    .outside(&Utc::now(), &Duration::seconds(TEMP_CODE_EXPIRED_AFTER))
            }) {
                return Ok(
                    (StatusCode::BAD_REQUEST, RequestErr::InvalidPasswordAndCode).into_response(),
                );
            };

            match purpose {
                TempCodePurpose::DeleteAccount => {
                    sqlx::query!("DELETE FROM users WHERE id = $1", user_id.as_ref())
                        .execute(&app_state.pool)
                        .await
                        .map_err(|e| {
                            tracing::error!("{e:?}");
                            InternalErr::DB(e)
                        })?;
                    app_state.session_cache.retain(|_, v| *v != user_id);
                    app_state.user_cache.remove(&user_id);
                }
                TempCodePurpose::ResetPassword => {
                    let Some(new_password) = req
                        .payload
                        .as_ref()
                        .filter(|new_password| is_strong(new_password))
                    else {
                        return Ok(
                            (StatusCode::BAD_REQUEST, RequestErr::WeakPassword).into_response()
                        );
                    };

                    sqlx::query!(
                        "UPDATE users
                        SET password_hash = $1,
                            updated_at = $2
                        WHERE id = $3",
                        hash_pass(new_password.as_bytes())?,
                        &now,
                        user_id.as_ref()
                    )
                    .execute(&app_state.pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("{e:?}");
                        InternalErr::DB(e)
                    })?;

                    app_state
                        .user_cache
                        .get_mut(&user_id)
                        .ok_or_else(|| {
                            let e = InternalErr::WriteCache("user".to_string());
                            tracing::error!("{e:?}");
                            e
                        })?
                        .value_mut()
                        .updated_at = Some(now);
                }
                TempCodePurpose::ValidateEmail => {
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
                        InternalErr::DB(e)
                    })?;

                    app_state
                        .user_cache
                        .get_mut(&user_id)
                        .ok_or_else(|| {
                            let e = InternalErr::WriteCache("user".to_string());
                            tracing::error!("{e:?}");
                            e
                        })?
                        .value_mut()
                        .verified_at = Some(now);
                }
            }
        }
    };

    Ok(StatusCode::OK.into_response())
}
