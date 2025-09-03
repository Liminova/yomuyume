use std::{fmt::Debug, num::ParseIntError};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{routes::user::MailerError, utils::archive_file::ArchiveFileError};

#[derive(Debug, thiserror::Error)]
pub enum InternalErr {
    #[error("database transaction error: {0}")]
    DBTransactionError(#[from] redb::TransactionError),
    #[error("database table error: {0}")]
    DBTableError(#[from] redb::TableError),
    #[error("database storage error: {0}")]
    DBStorageError(#[from] redb::StorageError),
    #[error("database commit error: {0}")]
    DBCommitError(#[from] redb::CommitError),

    #[error("IO error: {0}")]
    IO(std::io::Error),
    #[error("archive error: {0}")]
    Archive(ArchiveFileError),
    #[error("mailer error: {0}")]
    Mailer(#[from] MailerError),

    #[error("cache error: can't get {0} to read, this should not happen")]
    ReadCache(String),
    #[error("cache error: can't get {0} to write, this should not happen")]
    WriteCache(String),

    #[error("title w/ ID {0} has no chapter, this should not happen")]
    NoChapter(i64),
    #[error("chapter w/ ID {0} has no page, this should not happen")]
    ChapterNoPage(i64),

    #[error("can't generate secure id: {0}")]
    SecureID(#[from] argon2::password_hash::rand_core::Error),
    #[error("can't hash password: {0}")]
    PasswordHash(argon2::password_hash::errors::Error),
}

impl IntoResponse for InternalErr {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
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
    EmailAlreadyUsed,
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
}

impl IntoResponse for RequestErr {
    fn into_response(self) -> Response {
        format!("{self}").into_response()
    }
}
