use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use email_address::EmailAddress;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    models::prelude::{CustomID, *},
    routes::{hash_pass, Mailer},
    AppError, AppState,
};

/// Send an email to the user with a code to reset the password.
#[utoipa::path(get, path = "/api/user/reset", responses(
    (status = 200, description = "code sent to user's email"),
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

    let user_model = match Users::find()
        .filter(users::Column::Email.eq(email.to_string().to_ascii_lowercase()))
        .one(&app_state.db)
        .await
        .context("can't find user")?
    {
        Some(u) => u,
        None => return Ok((StatusCode::BAD_REQUEST, "user not found").into_response()),
    };

    if user_model.verified_at.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "user is not verified").into_response());
    }

    let temp_code_model = TempCodes::find()
        .filter(
            Condition::all()
                .add(temp_codes::Column::Purpose.eq(temp_codes::Purpose::ResetPassword))
                .add(temp_codes::Column::UserId.eq(&user_model.id)),
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
                user_id: Set(user_model.id),
                code: Set(code.clone()),
                purpose: Set(temp_codes::Purpose::ResetPassword),
                created_at: Set(Utc::now()),
            }
            .insert(&app_state.db)
            .await
            .context("can't insert temp code model")?;
        }
    }

    mailer
        .send(
            &user_model.username,
            &user_model.email,
            format!("{} - Reset your password", &app_state.config.app_name),
            format!(
                "Hello, {}!\n\n\
                You have requested to reset your password. Please copy the following code into the app to continue:\n\n\
                {}\n\n\
                If you did not request to reset your password, please ignore this email.\n\n\
                Best regards,\n\
                The {} team",
                &user_model.username,
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

/// The user provides the code received by email to confirm the password change.
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
    if query.new_password.is_empty() || query.code.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "password and code cannot be empty").into_response());
    }
    let code = match CustomID::from(query.code.clone()) {
        Ok(code) => code,
        Err(_) => return Ok((StatusCode::BAD_REQUEST, "invalid code").into_response()),
    };

    let temp_code_model = match TempCodes::find()
        .filter(
            Condition::all()
                .add(temp_codes::Column::Purpose.eq(temp_codes::Purpose::ResetPassword))
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

    let user_model = Users::find_by_id(&temp_code_model.user_id)
        .one(&app_state.db)
        .await
        .context("can't find user")?
        .ok_or_else(|| anyhow!("user not found"))?;

    let mut user_active: users::ActiveModel = user_model.into();
    user_active.password_hash = Set(hash_pass(query.new_password)?);
    user_active.updated_at = Set(Some(Utc::now()));
    user_active
        .update(&app_state.db)
        .await
        .context("can't update user")?;

    let temp_code_active: temp_codes::ActiveModel = temp_code_model.into();
    temp_code_active
        .delete(&app_state.db)
        .await
        .context("can't delete temp code model")?;

    Ok((StatusCode::OK).into_response())
}
