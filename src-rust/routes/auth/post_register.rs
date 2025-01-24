use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::hash_pass,
    utils::{app_error::AppError, app_state::AppState},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequestBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// register
#[utoipa::path(post, path = "api/auth/register", responses(
    (status = 200, description = "registration successful"),
    (status = 400, description = "bad request", body = String),
    (status = 409, description = "a conflict has occurred", body = String),
    (status = 500, description = "internal server error", body = String),
))]
pub async fn post_register(
    State(app_state): State<Arc<AppState>>,
    query: Json<RegisterRequestBody>,
) -> Result<Response, AppError> {
    if !email_address::EmailAddress::is_valid(&query.email) {
        return Ok((StatusCode::BAD_REQUEST, "invalid email").into_response());
    }

    if sqlx::query!(
        r#"SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE email = $1
            ) AS "exists!""#,
        query.email.to_string().to_ascii_lowercase()
    )
    .fetch_one(&app_state.pool)
    .await
    .context("can't query to check if email exists")?
    .exists
    {
        return Ok((
            StatusCode::CONFLICT,
            "a user with this email already exists",
        )
            .into_response());
    }

    let p = &query.password;
    let has_uppercase = p.chars().any(char::is_uppercase);
    let has_lowercase = p.chars().any(char::is_lowercase);
    let has_numeric = p.chars().any(char::is_numeric);
    let has_special = p.chars().any(|c| c.is_ascii_punctuation());
    let has_valid_length = p.len() >= 8 && p.len() <= 100;
    if !(has_uppercase && has_lowercase && has_numeric && has_special && has_valid_length) {
        return Ok((StatusCode::BAD_REQUEST, "password must be between 8 and 100 characters long and contain at least one uppercase letter, one lowercase letter, one number and one special character").into_response());
    }

    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)",
        app_state
            .id_generator
            .snowflake()
            .await
            .context("can't generate ID for record")?,
        query.username.as_str(),
        query.email.to_string().to_ascii_lowercase(),
        hash_pass(p)?
    )
    .execute(&app_state.pool)
    .await
    .context("can't insert user")?;

    Ok((StatusCode::OK).into_response())
}
