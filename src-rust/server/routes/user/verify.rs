use super::sendmail;
use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        prelude::*,
    },
    routes::{ErrRsp, GenericRsp},
    AppState,
};
use axum::{extract::State, response::IntoResponse, Extension};
use sea_orm::{ActiveModelTrait, Set};
use std::sync::Arc;

/// Send a verification email to the user's email address.
#[utoipa::path(get, path = "/api/user/verify", responses(
    (status = 200, description = "Verification email sent", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
))]
pub async fn get_verify(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<users::Model>,
) -> Result<impl IntoResponse, ErrRsp> {
    if user.is_verified {
        return Err(ErrRsp::bad_request("User is already verified."));
    }

    if data.env.smtp_host.is_none() {
        return Err(ErrRsp::internal(
            "SMTP is not configured, please contact the server administrator.",
        ));
    }

    let now = chrono::Utc::now();
    let token_claims = TokenClaims {
        sub: user.id,
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(1)).timestamp() as usize,
        purpose: Some(TokenClaimsPurpose::VerifyRegister),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .map_err(|e| ErrRsp::internal(format!("Failed to generate token. JWT error: {}", e)))?;

    let body = format!(
        "Hello {},\n\n\
        You have requested to verify your account. \
        Please click copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request this, please ignore this email.\n\n\
        Thanks,\n\
        The {} Team",
        &user.username, token, &data.env.app_name,
    );

    match sendmail(
        &data.env,
        &user.username,
        &user.email,
        &format!("{} - Verify your account", &data.env.app_name),
        &body,
    ) {
        Ok(_) => Ok(GenericRsp::create("Verification email sent.")),
        Err(e) => Err(ErrRsp::internal(format!(
            "Failed to send email. SMTP error: {}",
            e
        ))),
    }
}

/// The user provides the token received by email.
#[utoipa::path(post, path = "/api/user/verify", responses(
    (status = 200, description = "Account verification successful", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody),
    (status = 400, description = "Bad request", body = ErrorResponseBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
))]
pub async fn post_verify(
    State(data): State<Arc<AppState>>,
    Extension(purpose): Extension<TokenClaimsPurpose>,
    Extension(user): Extension<users::Model>,
) -> Result<impl IntoResponse, ErrRsp> {
    if user.is_verified {
        return Err(ErrRsp::bad_request("User is already verified."));
    }

    if purpose != TokenClaimsPurpose::VerifyRegister {
        return Err(ErrRsp::bad_request("Invalid request purpose."));
    }

    let mut user: users::ActiveModel = user.into();
    user.is_verified = Set(true);

    user.save(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't update user: {}", e)))?;

    Ok(GenericRsp::create("Account verification successful."))
}
