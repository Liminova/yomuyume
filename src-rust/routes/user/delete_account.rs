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

use crate::{
    models::prelude::*,
    routes::{check_pass, Mailer},
    AppError, AppState,
};

/// Send a request to delete the user.
///
/// The user will receive an email with a token to confirm the deletion.
#[utoipa::path(get, path = "/api/user/delete", responses(
    (status = 200, description = "Token sent to user's email"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 429, description = "Too many requests", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_delete_account(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
) -> Result<Response, AppError> {
    let mailer = Mailer::from(&app_state.config)?;

    let token = CustomID::new();

    let code_delete_account_active = if let Some(model) = CodeDeleteAccount::find_by_id(&user.id)
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find delete account: {}", e)))?
    {
        if model
            .created_at
            .checked_add_signed(chrono::Duration::minutes(5))
            .unwrap()
            .gt(&chrono::Utc::now())
        {
            return Ok((StatusCode::TOO_MANY_REQUESTS, "too many requests").into_response());
        }

        let mut active: code_delete_account::ActiveModel = model.into();
        active.user_id = NotSet;
        active.code = Set(token.clone());
        active.created_at = Set(chrono::Utc::now());
        active
    } else {
        code_delete_account::ActiveModel {
            user_id: Set(user.id),
            code: Set(token.clone()),
            created_at: Set(chrono::Utc::now()),
        }
    };
    code_delete_account_active
        .save(&app_state.db)
        .await
        .map_err(|e| {
            AppError::from(anyhow::anyhow!("can't save CodeDeleteAccount model: {}", e))
        })?;

    mailer.send(
        &user.username,
        &user.email,
        format!("{} - Delete your password", &app_state.config.app_name),
        format!(
            "Hello, {}!\n\n\
            // You have requested to delete your account. Please copy the following token into the app to continue:\n\n\
            {}\n\n\
            If you did not request to delete your account, please ignore this email.\n\n\
            Best regards,\n\
            The {} team",
            &user.username,
            &token,
            &app_state.config.app_name,
        ),
    ).map(|_| Ok((StatusCode::OK).into_response()))?
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DeleteRequestBody {
    pub code: String,
    pub password: String,
}

/// Confirm the deletion of the user.
///
/// The user will make a request with the token received by email.
#[utoipa::path(post, path = "/api/user/delete", responses(
    (status = 200, description = "User deleted"),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 429, description = "Too many requests", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn post_delete_account(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<DeleteRequestBody>,
) -> Result<Response, AppError> {
    /* #region - check request */
    if query.password.is_empty() || query.code.is_empty() {
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

    /* #region - get CodeDeleteAccount, check date and extract UserId */
    let user_id = match CodeDeleteAccount::find()
        .filter(code_delete_account::Column::Code.eq(&code))
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't find CodeDeleteAccount model: {}", e)))?
    {
        Some(model) => {
            let created_at = model.created_at;
            let user_id = model.user_id.clone();

            let active: code_delete_account::ActiveModel = model.into();
            active.delete(&app_state.db).await.map_err(|e| {
                AppError::from(anyhow::anyhow!(
                    "can't delete expired CodeDeleteAccount model: {}",
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
    }
    /* #endregion */

    if !check_pass(&user.password, &query.password) {
        return Ok((StatusCode::BAD_REQUEST, "invalid password").into_response());
    }

    let user: users::ActiveModel = user.into();
    user.delete(&app_state.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't delete user: {}", e)))?;

    Ok((StatusCode::OK).into_response())
}
