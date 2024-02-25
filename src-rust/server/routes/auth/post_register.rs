use std::sync::Arc;

pub use bridge::routes::auth::RegisterRequest;

use crate::{
    models::prelude::*,
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

/// Register a new user.
#[utoipa::path(post, path = "api/auth/register", responses(
    (status = 200, description = "Registration successful", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody),
    (status = 409, description = "A conflict has occurred", body = GenericResponseBody),
))]
pub async fn post_register(
    State(data): State<Arc<AppState>>,
    header: HeaderMap,
    query: Json<RegisterRequest>,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    if !email_address::EmailAddress::is_valid(&query.email) {
        return Err(builder.bad_request("Invalid email."));
    }

    let email_exists = Users::find()
        .filter(users::Column::Email.eq(&query.email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    if email_exists.is_some() {
        return Err(builder.conflict("An user with this email already exists."));
    }

    let password = &query.password;
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_numeric = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| c.is_ascii_punctuation());
    let has_valid_length = password.len() >= 8 && password.len() <= 100;
    if !(has_uppercase && has_lowercase && has_numeric && has_special && has_valid_length) {
        builder.bad_request("Password must be between 8 and 100 characters long and contain at least one uppercase letter, one lowercase letter, one number and one special character.");
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(query.password.as_bytes(), &salt)
        .map_err(|e| builder.internal(format!("Error while hashing password: {}", e)))?
        .to_string();

    let id = uuid::Uuid::new_v4().to_string();
    let username = query.username.to_string();
    let email = query.email.to_string().to_ascii_lowercase();
    let created_at = chrono::Utc::now().to_string();

    let user = users::ActiveModel {
        id: Set(id),
        username: Set(username.clone()),
        email: Set(email),
        created_at: Set(created_at.clone()),
        updated_at: Set(created_at),
        password: Set(hashed_password),
        is_verified: Set(false),
        ..Default::default()
    };

    user.insert(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?;

    let message = format!("User {} has been registered.", &username);

    Ok(builder.generic_success(message))
}
