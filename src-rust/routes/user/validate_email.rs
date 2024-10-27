use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{models::prelude::*, routes::Mailer, types::custom_id::CustomID, AppError, AppState};

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
    Extension(user): Extension<users::Model>,
) -> Result<Response, AppError> {
    if let Some(ref verified_at) = user.verified_at {
        return Ok((
            StatusCode::BAD_REQUEST,
            format!("user is already verified at {}.", verified_at),
        )
            .into_response());
    }
    let mailer = Mailer::from(&app_state.config)?;

    let temp_code_model = TempCodes::find()
        .filter(
            Condition::all()
                .add(temp_codes::Column::Purpose.eq(temp_codes::Purpose::ValidateEmail))
                .add(temp_codes::Column::UserId.eq(&user.id)),
        )
        .one(&app_state.db)
        .await
        .context("can't find temp code model")?;

    if let Some(ref model) = temp_code_model {
        if model
            .created_at
            .checked_add_signed(Duration::minutes(5))
            .map(|d| d.gt(&Utc::now()))
            .unwrap_or(true)
        {
            return Ok((StatusCode::TOO_MANY_REQUESTS).into_response());
        }
    }

    let code = CustomID::new();
    match temp_code_model {
        Some(model) => {
            let mut active: temp_codes::ActiveModel = model.into();
            active.id = NotSet;
            active.code = Set(code.clone());
            active.created_at = Set(Utc::now());
            active
                .update(&app_state.db)
                .await
                .context("can't update temp code model")?;
        }
        None => {
            temp_codes::ActiveModel {
                id: NotSet,
                user_id: Set(user.id),
                code: Set(code.clone()),
                purpose: Set(temp_codes::Purpose::ValidateEmail),
                created_at: Set(Utc::now()),
            }
            .insert(&app_state.db)
            .await
            .context("can't insert temp code model")?;
        }
    }

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
    Extension(user): Extension<users::Model>,
    Json(query): Json<ValidateEmailRequestBody>,
) -> Result<Response, AppError> {
    if let Some(ref verified_at) = user.verified_at {
        return Ok((
            StatusCode::BAD_REQUEST,
            format!("user is already verified at {}.", verified_at),
        )
            .into_response());
    }
    if query.code.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "token must not be empty").into_response());
    }
    let code = match CustomID::from(query.code.clone()) {
        Ok(code) => code,
        Err(_) => return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response()),
    };

    let temp_code_model = match TempCodes::find()
        .filter(
            Condition::all()
                .add(temp_codes::Column::Purpose.eq(temp_codes::Purpose::ValidateEmail))
                .add(temp_codes::Column::UserId.eq(&user.id))
                .add(temp_codes::Column::Code.eq(&code))
                .add(temp_codes::Column::CreatedAt.gt(Utc::now() - Duration::minutes(5))),
        )
        .one(&app_state.db)
        .await
        .context("can't find temp code model")?
    {
        Some(model) => model,
        None => return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response()),
    };

    let mut user_active: users::ActiveModel = user.into();
    user_active.verified_at = Set(Some(chrono::Utc::now()));
    user_active
        .update(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't save user: {}", e)))?;

    let temp_code_active: temp_codes::ActiveModel = temp_code_model.into();
    temp_code_active
        .delete(&app_state.db)
        .await
        .context("can't delete temp code model")?;

    Ok((StatusCode::OK).into_response())
}
