use std::sync::Arc;

pub use bridge::routes::user::ResetRequest;

use super::sendmail;
use crate::{
    models::{
        auth::{TokenClaims, TokenClaimsPurpose},
        prelude::*,
    },
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension, Json,
};
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

/// Send an email to the user with a token to reset the password.
#[utoipa::path(get, path = "/api/user/reset", responses(
    (status = 200, description = "Token sent to user's email", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
    (status = 409, description = "A conflict has occurred", body = GenericResponseBody),
))]
pub async fn get_reset(
    State(data): State<Arc<AppState>>,
    header: HeaderMap,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    if data.env.smtp_host.is_none() {
        return Err(
            builder.internal("SMTP is not configured, please contact the server administrator.")
        );
    }

    if !email_address::EmailAddress::is_valid(&email) {
        return Err(builder.bad_request("Invalid email."));
    }

    let user = Users::find()
        .filter(users::Column::Email.eq(&email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| builder.internal(format!("Can't find user: {}", e)))?
        .ok_or_else(|| builder.bad_request("User not found."))?;

    if !user.is_verified {
        return Err(builder.bad_request("User is not verified."));
    }

    let token = {
        let now = chrono::Utc::now();
        let token_claims = TokenClaims {
            sub: user.id.clone(),
            iat: now.timestamp() as usize,
            exp: (now
                + chrono::Duration::try_hours(1).ok_or_else(|| {
                    builder
                        .internal("Failed to generate token. Failed to calculate expiration time.")
                })?)
            .timestamp() as usize,
            purpose: Some(TokenClaimsPurpose::ResetPassword),
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &token_claims,
            &jsonwebtoken::EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
        )
    }
    .map_err(|e| builder.internal(format!("Failed to generate token. JWT error: {}", e)))?;

    let email = format!(
        "Hello, {}!\n\n\
        You have requested to reset your password. Please copy the following token into the app to continue:\n\n\
        {}\n\n\
        If you did not request to reset your password, please ignore this email.\n\n\
        Best regards,\n\
        The {} team",
        &user.username,
        token,
        &data.env.app_name,
    );

    match sendmail(
        &data.env,
        &user.username,
        &user.email,
        &format!("{} - Reset your password", &data.env.app_name),
        &email,
    ) {
        Ok(_) => Ok(builder.generic_success("Token sent to user's email.")),
        Err(e) => Err(builder.internal(format!("Failed to send email. SMTP error: {}", e))),
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
    header: HeaderMap,
    Json(query): Json<ResetRequest>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    if purpose != TokenClaimsPurpose::ResetPassword {
        return Err(builder.bad_request("Invalid request purpose."));
    }

    if query.password.is_empty() {
        return Err(builder.bad_request("Password cannot be empty."));
    }

    let user = Users::find()
        .filter(users::Column::Id.eq(user.id))
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .ok_or_else(|| builder.bad_request("User not found."))?;

    let password_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(query.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };
    if !password_valid {
        return Err(builder.bad_request("Invalid password."));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(query.password.as_bytes(), &salt)
        .map_err(|e| builder.internal(format!("Error while hashing password: {}", e)))?
        .to_string();

    let mut user: users::ActiveModel = user.into();
    user.password = Set(hashed_password);
    user.save(&data.db).await.map_err(|e| builder.db_error(e))?;

    Ok(builder.generic_success("Password reset successful."))
}
