use std::sync::Arc;

use super::sendmail;
use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        prelude::*,
    },
    AppError, AppState, GenericResponseBody,
};

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct ResetRequest {
    pub password: String,
}

/// Send an email to the user with a token to reset the password.
#[utoipa::path(get, path = "/api/user/reset", responses(
    (status = 200, description = "Token sent to user's email", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
    (status = 409, description = "A conflict has occurred", body = GenericResponseBody),
))]
pub async fn get_reset(
    State(data): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<Response, AppError> {
    if data.config.smtp_host.is_none() {
        return Err(AppError::from(anyhow::anyhow!(
            "SMTP is not configured, please contact the server administrator."
        )));
    }

    if !email_address::EmailAddress::is_valid(&email) {
        return Ok((StatusCode::BAD_REQUEST, "Invalid email.").into_response());
    }

    let user = match Users::find()
        .filter(users::Column::Email.eq(email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find user: {}", e)))?
    {
        Some(user) => user,
        None => return Ok((StatusCode::BAD_REQUEST, "User not found.").into_response()),
    };

    if !user.is_verified {
        return Ok((StatusCode::BAD_REQUEST, "User is not verified.").into_response());
    }

    let token = {
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
            purpose: Some(TokenClaimsPurpose::ResetPassword),
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &token_claims,
            &jsonwebtoken::EncodingKey::from_secret(data.config.jwt_secret.as_ref()),
        )
    }
    .map_err(|e| AppError::from(anyhow::anyhow!("Can't generate token: {}", e)))?;

    let email = format!(
        "Hello, {}!\n\n\
        You have requested to reset your password. Please copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request to reset your password, please ignore this email.\n\n\
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
        &format!("{} - Reset your password", &data.config.app_name),
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

/// The user provides the token received by email to confirm the password change.
#[utoipa::path(post, path = "/api/user/reset", responses(
    (status = 200, description = "Password reset successful", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
))]
pub async fn post_reset(
    State(data): State<Arc<AppState>>,
    Extension(purpose): Extension<TokenClaimsPurpose>,
    Extension(user): Extension<users::Model>,
    Json(query): Json<ResetRequest>,
) -> Result<Response, AppError> {
    if purpose != TokenClaimsPurpose::ResetPassword {
        return Ok((StatusCode::BAD_REQUEST, "Invalid request purpose.").into_response());
    }

    if query.password.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, "Password cannot be empty.").into_response());
    }

    let user = match Users::find()
        .filter(users::Column::Id.eq(user.id))
        .one(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't find user: {}", e)))?
    {
        Some(user) => user,
        None => return Ok((StatusCode::BAD_REQUEST, "User not found.").into_response()),
    };

    let password_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(query.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };
    if !password_valid {
        return Ok((StatusCode::BAD_REQUEST, "Invalid password.").into_response());
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(query.password.as_bytes(), &salt)
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't hash password: {}", e)))?
        .to_string();

    let mut user: users::ActiveModel = user.into();
    user.password = Set(hashed_password);
    user.save(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't save user: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("Password reset successful.")),
    )
        .into_response())
}
