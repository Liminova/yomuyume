use std::{fmt::Debug, num::ParseIntError};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::utils::{archive_file::ArchiveFileError, id_generator::GenerateIDErr};

#[derive(Debug, thiserror::Error)]
pub enum InternalErr {
    #[error("{0}")]
    DB(sqlx::Error),
    #[error("IO error: {0}")]
    IO(std::io::Error),
    #[error("archive error: {0}")]
    Archive(ArchiveFileError),
    #[error("mailer error: {0}")]
    Mailer(anyhow::Error),

    #[error("cache error: can't get {0} to read, this should not happen")]
    ReadCache(String),
    #[error("cache error: can't get {0} to write, this should not happen")]
    WriteCache(String),

    #[error("title w/ ID {0} has no chapter, this should not happen")]
    NoChapter(i64),
    #[error("chapter w/ ID {0} has no page, this should not happen")]
    ChapterNoPage(i64),

    #[error("can't generate snowflake id: {0}")]
    Snowflake(GenerateIDErr),
    #[error("can't generate secure id: {0}")]
    SecureID(argon2::password_hash::rand_core::Error),
    #[error("can't hash password: {0}")]
    PasswordHash(argon2::password_hash::errors::Error),

    #[error("can't set live config: {0}")]
    LiveConfig(String),
}

impl IntoResponse for InternalErr {
    fn into_response(self) -> Response {
        tracing::error!("In case I forgot to call tracing::error: {self:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}", anyhow::anyhow!("{self}")),
        )
            .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestErr {
    #[error("missing session-id cookie")]
    MissingSessionID,
    #[error("can't parse session id: {0}")]
    CantParseSessionID(ParseIntError),
    #[error("missing session secret cookie")]
    MissingSessionSecret,
    #[error("invalid session")]
    InvalidSession,
    #[error("session expired")]
    SessionExpired,

    #[error("you don't even logged in")]
    YouDontEvenLoggedIn,
    #[error("invalid username or password")]
    InvalidCredentials,
    #[error("invalid email")]
    InvalidEmail,
    #[error("email is already used")]
    SomeoneUseThisEmail,
    #[error(
        "password must be between 8 and 100 characters long and contain at least one uppercase letter, one lowercase letter, one number and one special character"
    )]
    WeakPassword,

    #[error("invalid current password")]
    InvalidCurrentPassword,

    #[error("invalid code")]
    InvalidCode,
    #[error("expired code")]
    ExpiredCode,

    #[error("your email is already verified")]
    AlreadyVerified,

    #[error(
        "set live config do nothing, this might because the whole body is empty, or the backend forgot to handle this case"
    )]
    SetLiveConfigDoNothing,
}

impl IntoResponse for RequestErr {
    fn into_response(self) -> Response {
        format!("{}", anyhow::anyhow!("{self}")).into_response()
    }
}
