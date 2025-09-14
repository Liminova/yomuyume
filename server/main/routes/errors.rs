use std::fmt::Debug;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    routes::user::MailerError,
    utils::{archive_file::ArchiveFileError, pathbuf_utils::ReadCategoryInfoError},
};

#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("archive error: {0}")]
    Archive(#[from] ArchiveFileError),
    #[error("mailer error: {0}")]
    Mailer(#[from] MailerError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("can't hash password: {0}")]
    PasswordHash(argon2::password_hash::errors::Error),

    #[error("can't read category info: {0}")]
    ReadCategoryInfo(#[from] ReadCategoryInfoError),
}

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("missing session secret cookie")]
    MissingSessionSecret,

    #[error("invalid username or password")]
    InvalidCredentials,
    #[error("invalid email")]
    InvalidEmail,
    #[error("malformed request, either both 'code' and 'new_password' are provided or none")]
    MailformedForgotPasswordRequest,
    #[error("email is already used")]
    EmailAlreadyUsed,
    #[error(
        "password must be between 8 and 100 characters long and contain at least one uppercase letter, one lowercase letter, one number and one special character"
    )]
    WeakPassword,

    #[error("current password is required to change password")]
    CurrentPasswordRequired,
    #[error("invalid current password")]
    InvalidCurrentPassword,

    #[error("invalid code")]
    InvalidCode,
    #[error("expired code")]
    ExpiredCode,
    #[error("no forgot password request found")]
    NoForgotPasswordRequestFound,
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        format!("{self}").into_response()
    }
}
