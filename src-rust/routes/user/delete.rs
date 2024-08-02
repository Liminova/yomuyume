use std::sync::Arc;

use super::{check_pass, sendmail};
use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        prelude::*,
    },
    AppError, AppState, GenericResponseBody,
};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DeleteRequestBody {
    pub password: String,
}

/// Send a request to delete the user.
///
/// The user will receive an email with a token to confirm the deletion.
#[utoipa::path(get, path = "/api/user/delete", responses(
    (status = 200, description = "Token sent to user's email", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = String),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn get_delete(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
) -> Result<Response, AppError> {
    if data.config.smtp_host.is_none() {
        return Err(AppError::from(anyhow::anyhow!(
            "SMTP is not configured, please contact the server administrator.",
        )));
    }

    let now = chrono::Utc::now();
    let token_claims = TokenClaims {
        sub: user.id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now
            + chrono::Duration::try_hours(1).ok_or_else(|| {
                AppError::from(anyhow::anyhow!(
                    "Can't generate token: can't calculate expiration time."
                ))
            })?)
        .timestamp() as usize,
        purpose: Some(TokenClaimsPurpose::DeleteAccount),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(data.config.jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::from(anyhow::anyhow!("Can't generate token: {}", e)))?;

    let email = format!(
        "Hello, {}!\n\n\
        // You have requested to delete your account. Please copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request to delete your account, please ignore this email.\n\n\
        Best regards,\n\
        The {} team",
        &user.username,
        token,
        &data.config.app_name,
    );

    match sendmail(
        &data.config,
        &user.username,
        &user.email,
        &format!("{} - Delete your password", &data.config.app_name),
        &email,
    ) {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(GenericResponseBody::new("Token sent to user's email.")),
        )
            .into_response()),
        Err(e) => Err(AppError::from(anyhow::anyhow!("Can't send email: {}", e))),
    }
}

/// Confirm the deletion of the user.
///
/// The user will make a request with the token received by email.
#[utoipa::path(post, path = "/api/user/delete", responses(
    (status = 200, description = "User deleted", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = String),
    (status = 400, description = "Bad request", body = String),
    (status = 401, description = "Unauthorized", body = String),
))]
pub async fn post_delete(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<DeleteRequestBody>,
) -> Result<Response, AppError> {
    if query.password.is_empty() {
        return Err(AppError::from(anyhow::anyhow!("Password cannot be empty.")));
    }

    if !check_pass(&user.password, &query.password) {
        return Err(AppError::from(anyhow::anyhow!(
            "Invalid username or password."
        )));
    }

    let user = match Users::find()
        .filter(users::Column::Id.eq(user.id))
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find user: {}", e)))?
    {
        Some(user) => user,
        None => return Err(AppError::from(anyhow::anyhow!("Invalid user."))),
    };

    let user: users::ActiveModel = user.into();

    user.delete(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't delete user: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("User deleted.")),
    )
        .into_response())
}
