use std::sync::Arc;

use super::sendmail;
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
use sea_orm::{ActiveModelTrait, Set};

/// Send a verification email to the user's email address.
#[utoipa::path(get, path = "/api/user/verify", responses(
    (status = 200, description = "Verification email sent", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
))]
pub async fn get_verify(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
) -> Result<Response, AppError> {
    if user.is_verified {
        return Ok((StatusCode::BAD_REQUEST, "User is already verified.").into_response());
    }

    if data.config.smtp_host.is_none() {
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "SMTP is not configured.").into_response());
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
        purpose: Some(TokenClaimsPurpose::VerifyRegister),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(data.config.jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::from(anyhow::anyhow!("Can't generate token: {}", e)))?;

    let body = format!(
        "Hello {},\n\n\
        You have requested to verify your account. \
        Please click copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request this, please ignore this email.\n\n\
        Thanks,\n\
        The {} Team",
        &user.username, token, &data.config.app_name,
    );

    match sendmail(
        &data.config,
        &user.username,
        &user.email,
        &format!("{} - Verify your account", &data.config.app_name),
        &body,
    ) {
        Ok(_) => Ok((
            StatusCode::OK,
            Json(GenericResponseBody::new("Verification email sent.")),
        )
            .into_response()),
        Err(e) => Err(AppError::from(anyhow::anyhow!("Can't send email: {}", e))),
    }
}

/// The user provides the token received by email.
#[utoipa::path(post, path = "/api/user/verify", responses(
    (status = 200, description = "Account verification successful", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
    (status = 400, description = "Bad request", body = GenericResponseBody),
    (status = 401, description = "Unauthorized", body = GenericResponseBody),
))]
pub async fn post_verify(
    State(data): State<Arc<AppState>>,
    Extension(purpose): Extension<TokenClaimsPurpose>,
    Extension(user): Extension<users::Model>,
) -> Result<Response, AppError> {
    if user.is_verified {
        return Ok((StatusCode::BAD_REQUEST, "User is already verified.").into_response());
    }

    if purpose != TokenClaimsPurpose::VerifyRegister {
        return Ok((StatusCode::BAD_REQUEST, "Invalid request purpose.").into_response());
    }

    let mut user: users::ActiveModel = user.into();
    user.is_verified = Set(true);

    user.save(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("Can't save user: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(GenericResponseBody::new("Account verification successful.")),
    )
        .into_response())
}
