use std::fmt::Debug;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::utils::{archive_file::ArchiveFileError, id_generator::GenerateIDErr};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    DB(sqlx::Error),
    #[error("io error: {0}")]
    IO(std::io::Error),
    #[error("archive error: {0}")]
    Archive(ArchiveFileError),
    #[error("mailer error: {0}")]
    Mailer(anyhow::Error),

    #[error("cache error: can't get {0} to read, this should not happen")]
    ReadCache(String),
    #[error("cache error: can't get {0} to write, this should not happen")]
    WriteCache(String),

    #[error("can't generate snowflake id: {0}")]
    Snowflake(GenerateIDErr),
    #[error("can't hash password: {0}")]
    PasswordHash(argon2::password_hash::errors::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self:?}")).into_response()
    }
}
