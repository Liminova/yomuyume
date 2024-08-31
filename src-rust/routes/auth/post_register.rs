use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::{models::prelude::*, routes::hash_pass, AppError, AppState};

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct RegisterRequestBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Register a new user.
#[utoipa::path(post, path = "api/auth/register", responses(
    (status = 200, description = "Registration successful"),
    (status = 500, description = "Internal server error", body = String),
    (status = 409, description = "A conflict has occurred", body = String),
))]
pub async fn post_register(
    State(data): State<Arc<AppState>>,
    query: Json<RegisterRequestBody>,
) -> Result<Response, AppError> {
    if !email_address::EmailAddress::is_valid(&query.email) {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
    }

    let email_exists = Users::find()
        .filter(users::Column::Email.eq(query.email.to_string().to_ascii_lowercase()))
        .one(&data.db)
        .await
        .map_err(AppError::from)?;

    if email_exists.is_some() {
        return Ok((
            StatusCode::CONFLICT,
            "a user with this email already exists",
        )
            .into_response());
    }

    let p = &query.password;
    let has_uppercase = p.chars().any(|c| c.is_uppercase());
    let has_lowercase = p.chars().any(|c| c.is_lowercase());
    let has_numeric = p.chars().any(|c| c.is_numeric());
    let has_special = p.chars().any(|c| c.is_ascii_punctuation());
    let has_valid_length = p.len() >= 8 && p.len() <= 100;
    if !(has_uppercase && has_lowercase && has_numeric && has_special && has_valid_length) {
        return Ok((StatusCode::BAD_REQUEST, "password must be between 8 and 100 characters long and contain at least one uppercase letter, one lowercase letter, one number and one special character").into_response());
    }

    let username = query.username.to_string();
    let email = query.email.to_string().to_ascii_lowercase();
    let created_at = chrono::Utc::now();

    let user = users::ActiveModel {
        id: Set(UserID::new()),
        username: Set(username.clone()),
        email: Set(email),
        created_at: Set(created_at.clone()),
        updated_at: Set(created_at),
        password: Set(hash_pass(p)?),
        is_verified: Set(false),
        ..Default::default()
    };

    user.insert(&data.db)
        .await
        .map_err(|e| AppError::from(anyhow::anyhow!("can't insert user: {}", e)))?;

    Ok((StatusCode::OK).into_response())
}
