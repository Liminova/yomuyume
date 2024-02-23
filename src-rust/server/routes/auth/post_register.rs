use crate::{
    models::prelude::*,
    routes::{ErrRsp, GenericRsp},
    AppState,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand_core::OsRng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Register a new user.
#[utoipa::path(post, path = "/api/auth/register", responses(
    (status = 200, description = "Registration successful", body = GenericResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody),
    (status = 409, description = "A conflict has occurred", body = ErrorResponseBody),
))]
pub async fn post_register(
    State(data): State<Arc<AppState>>,
    query: Json<RegisterRequest>,
) -> Result<impl IntoResponse, ErrRsp> {
    if !email_address::EmailAddress::is_valid(&query.email) {
        return Err(ErrRsp::bad_request("Invalid email."));
    }

    let email_exists = Users::find()
        .filter(users::Column::Email.eq(&query.email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(|e| ErrRsp::internal(format!("Can't fetch user from DB: {}", e)))?;

    if email_exists.is_some() {
        return Err(ErrRsp::new(
            StatusCode::CONFLICT,
            "An user with this email already exists.",
        ));
    }

    let password = &query.password;
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_numeric = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| c.is_ascii_punctuation());
    let has_valid_length = password.len() >= 8 && password.len() <= 100;
    if !(has_uppercase && has_lowercase && has_numeric && has_special && has_valid_length) {
        return Err(ErrRsp::bad_request(
            "Password must be between 8 and 100 characters long and contain at least one uppercase letter, one lowercase letter, one number and one special character.",
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(query.password.as_bytes(), &salt)
        .map_err(|e| ErrRsp::internal(format!("Error while hashing password: {}", e)))
        .map(|hash| hash.to_string())?;

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
        .map_err(|e| ErrRsp::internal(format!("Can't insert user to DB: {}", e)))?;

    let message = format!("User {} has been registered.", &username);
    Ok(GenericRsp::create(message))
}
