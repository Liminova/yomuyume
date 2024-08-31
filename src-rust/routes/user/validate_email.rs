use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::{models::prelude::*, routes::mailer::Mailer, AppError, AppState};

/// Send a verification email to the user's email address.
#[utoipa::path(get, path = "/api/user/verify", responses(
    (status = 200, description = "Verification email sent"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 429, description = "Too many requests", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_validate_email(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
) -> Result<Response, AppError> {
    if user.is_verified {
        return Ok((StatusCode::BAD_REQUEST, "user is already verified").into_response());
    }
    let mailer = Mailer::from(&app_state.config)?;

    let code = CustomID::new();
    let code_validate_email_active = match CodeValidateEmail::find_by_id(&user.id)
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find verify email: {}", e)))?
    {
        Some(model) => {
            if model
                .created_at
                .checked_add_signed(chrono::Duration::minutes(5))
                .unwrap()
                .gt(&chrono::Utc::now())
            {
                return Ok((StatusCode::TOO_MANY_REQUESTS, "too many requests").into_response());
            }
            let mut active: code_validate_email::ActiveModel = model.into();
            active.user_id = NotSet;
            active.code = Set(code.clone());
            active.created_at = Set(chrono::Utc::now());
            active
        }
        None => code_validate_email::ActiveModel {
            user_id: Set(user.id),
            code: Set(code.clone()),
            created_at: Set(chrono::Utc::now()),
        },
    };
    code_validate_email_active
        .save(&app_state.db)
        .await
        .map_err(|e| {
            AppError::from(anyhow::anyhow!("can't save CodeValidateEmail model: {}", e))
        })?;

    mailer
        .send(
            &user.username,
            &user.email,
            format!("{} - Verify your account", &app_state.config.app_name),
            format!(
                "Hello {},\n\n\
            You have requested to verify your account. \
            Please click copy the following token into the app to continue:\n\n\
            {}\n\n\
            If you did not request this, please ignore this email.\n\n\
            Thanks,\n\
            The {} Team",
                &user.username, &code, &app_state.config.app_name,
            ),
        )
        .map(|_| Ok((StatusCode::OK).into_response()))?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ValidateEmailRequestBody {
    pub code: String,
}

/// The user provides the token received by email.
#[utoipa::path(post, path = "/api/user/verify", responses(
    (status = 200, description = "Account verification successful"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn post_validate_email(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<ValidateEmailRequestBody>,
) -> Result<Response, AppError> {
    /* #region - check request */
    if user.is_verified {
        return Ok((StatusCode::BAD_REQUEST, "User is already verified.").into_response());
    }
    if query.code.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "token must not be empty").into_response());
    }
    /* #endregion */

    /* #region - get CodeValidateEmail, check date, and get the user_id */
    let user_id = match CodeValidateEmail::find()
        .filter(code_validate_email::Column::Code.eq(&query.code))
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find CodeValidateEmail model: {}", e)))?
    {
        Some(model) => {
            let created_at = model.created_at;
            let user_id = model.user_id.clone();

            let active: code_validate_email::ActiveModel = model.into();
            active.delete(&app_state.db).await.map_err(|e| {
                AppError::from(anyhow::anyhow!(
                    "can't delete expired CodeValidateEmail model: {}",
                    e
                ))
            })?;

            if created_at
                .checked_add_signed(chrono::Duration::minutes(5))
                .unwrap()
                .le(&chrono::Utc::now())
            {
                return Ok((StatusCode::BAD_REQUEST, "token expired").into_response());
            }

            user_id
        }
        None => return Ok((StatusCode::BAD_REQUEST, "invalid token").into_response()),
    };
    /* #endregion */

    /* #region - make sure it's come from the same user */
    if user_id != user.id {
        return Ok((
            StatusCode::BAD_REQUEST,
            "request not come from the same user",
        )
            .into_response());
    };
    /* #endregion */

    let mut active: users::ActiveModel = user.into();
    active.is_verified = Set(true);
    active
        .update(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't save user: {}", e)))?;

    Ok((StatusCode::OK).into_response())
}
