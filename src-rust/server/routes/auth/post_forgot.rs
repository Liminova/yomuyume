use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::{
        errors::{InternalErr, RequestErr},
        hash_pass, is_strong,
        user::Mailer,
    },
    structs::db_enums::TempCodePurpose as CodePurpose,
    traits::chrono_utils::ChronoUtils,
    utils::{
        app_state::AppState,
        constants::{FORGOT_PATH, TEMP_CODE_EXPIRED_AFTER, TEMP_CODE_REQUEST_RATE_LIMIT},
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
#[utoipa::path(post, path = FORGOT_PATH, responses(
    (status = 200, description = "Request successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 418, description = "What are you trying to do?",),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn post_forgot(
    State(app_state): State<Arc<AppState>>,
    query: Json<ForgotRequest>,
) -> Result<Response, InternalErr> {
    let now = Utc::now();

    match (query.code.as_deref(), query.new_password.as_deref()) {
        (None, None) => {
            let mailer = Mailer::from(&app_state.config).map_err(|e| {
                tracing::error!("{e}");
                InternalErr::Mailer(e)
            })?;

            // too many request
            let Some((user_id, created_at)) = sqlx::query!(
                r#"SELECT u.id AS user_id,
                    tc.created_at AS "created_at?"
                FROM users u
                    LEFT JOIN temp_codes tc ON u.id = tc.user_id
                    AND tc.purpose = $1
                WHERE u.email = $2
                LIMIT 1"#,
                CodePurpose::ResetPassword as CodePurpose,
                &query.email,
            )
            .fetch_optional(&app_state.pool)
            .await
            .map(|r| r.map(|r| (r.user_id, r.created_at)))
            .map_err(|e| {
                tracing::error!("{e}");
                InternalErr::DB(e)
            })?
            else {
                return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidEmail).into_response());
            };
            if created_at
                .is_some_and(|r| r.inside(&now, &Duration::seconds(TEMP_CODE_REQUEST_RATE_LIMIT)))
            {
                return Ok(StatusCode::TOO_MANY_REQUESTS.into_response());
            }

            let code = sqlx::query!(
                "INSERT INTO temp_codes (id, purpose, user_id, code, created_at)
                VALUES ($1, $2, $3, $4, $5) ON CONFLICT (purpose, user_id) DO
                UPDATE
                SET created_at = $5
                RETURNING code",
                app_state.id_generator.snowflake().await.map_err(|e| {
                    tracing::error!("{e}");
                    InternalErr::Snowflake(e)
                })?,
                CodePurpose::ResetPassword as CodePurpose,
                &user_id,
                app_state.id_generator.secure(),
                now
            )
            .fetch_one(&app_state.pool)
            .await
            .map_err(|e| {
                tracing::error!("{e}");
                InternalErr::DB(e)
            })?
            .code;

            let (username, email) = app_state
                .user_cache
                .get(&user_id.into())
                .ok_or_else(|| {
                    let e = InternalErr::ReadCache("user".to_string());
                    tracing::error!("{e}");
                    e
                })
                .map(|u| (u.username.clone(), u.email.clone()))?;
            let app_name = &app_state.config.app_name;

            mailer.send(
                &username,
                &email,
                format!("{app_name} - forgot password"),
                format!(
                    "Hello, {username}!\n\n\
                    You have requested to reset your password. Please copy the following code into the app to continue:\n\n\
                    {code}\n\n\
                    If you don't recognize this action or don't own this account, ignore this email.\n\n\
                    Best regards,\n\
                    The {app_name} team",
                ),
            )
            .map_err(|e| {
                tracing::error!("{e}");
                InternalErr::Mailer(e)
            })?;
        }

        (Some(code), Some(new_password)) => {
            let Some((user_id, created_at)) = sqlx::query!(
                r#"WITH u AS (
                    SELECT u.id AS id
                    FROM users u
                    WHERE u.email = $1
                ),
                deleted_code AS (
                    DELETE FROM temp_codes tc
                    WHERE tc.code = $2
                        AND tc.purpose = $3
                        AND tc.user_id IN (
                            SELECT id
                            FROM u
                        )
                    RETURNING tc.created_at
                )
                SELECT u.id as user_id,
                    COALESCE(dc.created_at, null) as created_at
                FROM u
                    LEFT JOIN deleted_code dc ON true"#,
                &query.email,
                &code,
                CodePurpose::ResetPassword as CodePurpose,
            )
            .fetch_optional(&app_state.pool)
            .await
            .map_err(|e| {
                tracing::error!("{e}");
                InternalErr::DB(e)
            })?
            .map(|r| (r.user_id, r.created_at)) else {
                return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidEmail).into_response());
            };

            if let Some(created_at) = created_at {
                if created_at.outside(&now, &Duration::seconds(TEMP_CODE_EXPIRED_AFTER)) {
                    return Ok((StatusCode::BAD_REQUEST, RequestErr::ExpiredCode).into_response());
                }
            } else {
                return Ok((StatusCode::BAD_REQUEST, RequestErr::InvalidCode).into_response());
            }

            if !is_strong(new_password) {
                return Ok((StatusCode::BAD_REQUEST, RequestErr::WeakPassword).into_response());
            }

            sqlx::query!(
                "UPDATE users
                SET password_hash = $1,
                    updated_at = $2
                WHERE id = $3",
                hash_pass(new_password.as_bytes()).map_err(|e| {
                    tracing::error!("{e}");
                    e
                })?,
                &now,
                user_id
            )
            .execute(&app_state.pool)
            .await
            .map_err(|e| {
                tracing::error!("{e}");
                InternalErr::DB(e)
            })?;

            if let Some(mut u) = app_state.user_cache.get_mut(&user_id.into()) {
                u.updated_at = Some(now);
            }
        }

        _ => return Ok(StatusCode::IM_A_TEAPOT.into_response()),
    }

    Ok(StatusCode::OK.into_response())
}
