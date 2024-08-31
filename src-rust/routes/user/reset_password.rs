use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use email_address::EmailAddress;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::{
    models::prelude::{CustomID, *},
    routes::{hash_pass, Mailer},
    AppError, AppState,
};

/// Send an email to the user with a token to reset the password.
#[utoipa::path(get, path = "/api/user/reset", responses(
    (status = 200, description = "Token sent to user's email"),
    (status = 400, description = "Bad request", body = String),
    (status = 429, description = "Too many requests", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_reset_password(
    State(app_state): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<Response, AppError> {
    if !EmailAddress::is_valid(&email) {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
    }
    let mailer = Mailer::from(&app_state.config)?;

    let user = match Users::find()
        .filter(users::Column::Email.eq(email.to_string().to_ascii_lowercase()))
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find user {}", e)))?
    {
        Some(user) => user,
        None => return Ok((StatusCode::BAD_REQUEST, "user not found").into_response()),
    };
    if !user.is_verified {
        return Ok((StatusCode::BAD_REQUEST, "user is not verified").into_response());
    }

    let token = CustomID::new();

    let code_reset_pass_active = if let Some(model) = CodeResetPassword::find_by_id(&user.id)
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find reset password: {}", e)))?
    {
        if model
            .created_at
            .checked_add_signed(chrono::Duration::minutes(5))
            .unwrap()
            .gt(&chrono::Utc::now())
        {
            return Ok((StatusCode::TOO_MANY_REQUESTS, "too many requests").into_response());
        }

        let mut active: code_reset_password::ActiveModel = model.into();
        active.user_id = NotSet;
        active.code = Set(token.clone());
        active.created_at = Set(chrono::Utc::now());
        active
    } else {
        code_reset_password::ActiveModel {
            user_id: Set(user.id),
            code: Set(token.clone()),
            created_at: Set(chrono::Utc::now()),
        }
    };
    code_reset_pass_active
        .save(&app_state.db)
        .await
        .map_err(|e| {
            AppError::from(anyhow::anyhow!("can't save CodeResetPassword model: {}", e))
        })?;

    mailer
        .send(
            &user.username,
            &user.email,
            format!("{} - Reset your password", &app_state.config.app_name),
            format!(
                "Hello, {}!\n\n\
                You have requested to reset your password. Please copy the following token into the app to continue:\n\n\
                {}\n\n\
                If you did not request to reset your password, please ignore this email.\n\n\
                Best regards,\n\
                The {} team",
                &user.username,
                &token,
                &app_state.config.app_name,
            ),
        )
        .map(|_| Ok((StatusCode::OK).into_response()))?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResetRequestBody {
    pub code: String,
    pub new_password: String,
}

/// The user provides the token received by email to confirm the password change.
#[utoipa::path(post, path = "/api/user/reset", responses(
    (status = 200, description = "Password reset successful"),
    (status = 500, description = "Internal server error", body = String),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn post_reset_password(
    State(app_state): State<Arc<AppState>>,
    Json(query): Json<ResetRequestBody>,
) -> Result<Response, AppError> {
    /* #region - check request */
    if query.new_password.is_empty() || query.code.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "password and token cannot be empty",
        )
            .into_response());
    }
    let code = match CustomID::from(query.code.clone()) {
        Ok(code) => code,
        Err(_) => return Ok((StatusCode::BAD_REQUEST, "invalid token").into_response()),
    };
    /* #endregion */

    /* #region get CodeResetModel, check date, and get the user_id */
    let user_id = match CodeResetPassword::find()
        .filter(code_reset_password::Column::Code.eq(&code))
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find CodeResetPassword model: {}", e)))?
    {
        Some(model) => {
            let created_at = model.created_at;
            let user_id = model.user_id.clone();

            let active: code_reset_password::ActiveModel = model.into();
            active.delete(&app_state.db).await.map_err(|e| {
                AppError::from(anyhow::anyhow!(
                    "can't delete expired CodeResetPassword model: {}",
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

    let user_model = match Users::find_by_id(&user_id)
        .one(&app_state.db)
        .await
        .map_err(|e| {
            AppError::from(anyhow::anyhow!(
                "can't get User model from CodeResetPassword model: {}",
                e
            ))
        })? {
        Some(model) => model,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                "password reset request made by a deleted user",
            )
                .into_response());
        }
    };

    let mut user_active: users::ActiveModel = user_model.into();
    user_active.password = Set(hash_pass(query.new_password)?);
    user_active
        .save(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't save user: {}", e)))?;

    Ok((StatusCode::OK).into_response())
}
