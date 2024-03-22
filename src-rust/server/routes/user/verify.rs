use std::sync::Arc;

use super::sendmail;
use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        prelude::*,
    },
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{extract::State, http::HeaderMap, response::IntoResponse, Extension};
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
    header: HeaderMap,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    if user.is_verified {
        return Err(builder.bad_request("User is already verified."));
    }

    if data.env.smtp_host.is_none() {
        return Err(
            builder.internal("SMTP is not configured, please contact the server administrator.")
        );
    }

    let now = chrono::Utc::now();
    let token_claims = TokenClaims {
        sub: user.id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now
            + chrono::Duration::try_hours(1).ok_or_else(|| {
                builder.internal("Failed to generate token. Failed to calculate expiration time.")
            })?)
        .timestamp() as usize,
        purpose: Some(TokenClaimsPurpose::VerifyRegister),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_claims,
        &jsonwebtoken::EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .map_err(|e| {
        // GenericResponse::internal(format!("Failed to generate token. JWT error: {}", e))
        builder.internal(format!("Failed to generate token. JWT error: {}", e))
    })?;

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
        Ok(_) => Ok(builder.generic_success("Verification email sent.")),
        Err(e) => Err(builder.internal(format!("Failed to send email. SMTP error: {}", e))),
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
    header: HeaderMap,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    if user.is_verified {
        return Err(builder.bad_request("User is already verified."));
    }

    if purpose != TokenClaimsPurpose::VerifyRegister {
        return Err(builder.bad_request("Invalid request purpose."));
    }

    let mut user: users::ActiveModel = user.into();
    user.is_verified = Set(true);

    user.save(&data.db).await.map_err(|e| builder.db_error(e))?;

    Ok(builder.generic_success("Account verification successful."))
}
