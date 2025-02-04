use std::{fmt::Debug, num::ParseIntError};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::utils::{archive_file::ArchiveFileError, id_generator::GenerateIDErr};

#[derive(Debug, thiserror::Error)]
pub enum InternalErr {
    #[error("DB error: {0}")]
    DB(sqlx::Error),
    #[error("IO error: {0}")]
    IO(std::io::Error),
    #[error("Archive error: {0}")]
    Archive(ArchiveFileError),
    #[error("Mailer error: {0}")]
    Mailer(anyhow::Error),

    #[error("Cache error: can't get {0} to read, this should not happen")]
    ReadCache(String),
    #[error("Cache error: can't get {0} to write, this should not happen")]
    WriteCache(String),

    #[error("Title w/ ID {0} has no chapter, this should not happen")]
    NoChapter(i64),
    #[error("Title w/ ID {0} has no page, this should not happen")]
    TitleNoPage(i64),
    #[error("Chapter w/ ID {0} has no page, this should not happen")]
    ChapterNoPage(i64),

    #[error("Can't generate snowflake id: {0}")]
    Snowflake(GenerateIDErr),
    #[error("Can't hash password: {0}")]
    PasswordHash(argon2::password_hash::errors::Error),

    #[error("Can't set live config: {0}")]
    LiveConfig(String),
}

impl IntoResponse for InternalErr {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}", anyhow::anyhow!("{self:?}")),
        )
            .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestErr {
    #[error("Missing session-id cookie")]
    MissingSessionID,
    #[error("Can't parse session id: {0}")]
    CantParseSessionID(ParseIntError),
    #[error("Missing session secret cookie")]
    MissingSessionSecret,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Session expired")]
    SessionExpired,

    #[error("You don't even logged in")]
    YouDontEvenLoggedIn,
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Invalid email")]
    InvalidEmail,
    #[error("Email is already used")]
    SomeoneUseThisEmail,
    #[error("Password must be between 8 and 100 characters long and contain at least one uppercase letter, one lowercase letter, one number and one special character")]
    WeakPassword,

    #[error("Invalid password and code")]
    InvalidPasswordAndCode,
    #[error("Invalid current password")]
    InvalidCurrentPassword,

    #[error("Your email is already verified")]
    AlreadyVerified,
}

impl IntoResponse for RequestErr {
    fn into_response(self) -> Response {
        format!("{}", anyhow::anyhow!("{self}")).into_response()
    }
}
