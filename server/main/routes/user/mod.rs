mod get_whoami;
mod post_modify;
mod put_progress;

pub use get_whoami::*;
pub use post_modify::*;
pub use put_progress::*;

use lettre::{
    Address, Message, SmtpTransport, Transport,
    address::AddressError,
    message::{IntoBody, Mailbox, header::ContentType},
    transport::smtp::{self, authentication::Credentials, response::Response as LetterResponse},
};

use crate::config::Smtp;

pub struct Mailer {
    sender: Mailbox,
    mailer: SmtpTransport,
}

#[derive(Debug, thiserror::Error)]
pub enum MailerError {
    #[error("invalid email: {0}")]
    InvalidEmail(#[from] AddressError),

    #[error("email transport error: {0}")]
    TransportError(#[from] smtp::Error),

    #[error("other error: {0}")]
    Other(#[from] lettre::error::Error),
}

impl Mailer {
    pub fn from(smtp: &Smtp) -> Result<Mailer, MailerError> {
        Ok(Mailer {
            mailer: SmtpTransport::relay(&smtp.host)
                .map_err(MailerError::TransportError)?
                .credentials(Credentials::new(
                    smtp.username.clone(),
                    smtp.password.clone(),
                ))
                .build(),
            sender: Mailbox {
                name: Some(smtp.from_name.clone()),
                email: smtp.from_email.clone(),
            },
        })
    }

    pub fn send(
        &self,
        receiver_name: Option<String>,
        receiver_email: Address,
        subject: impl AsRef<str>,
        body: impl IntoBody,
    ) -> Result<LetterResponse, MailerError> {
        Ok(self.mailer.send(
            &Message::builder()
                .from(self.sender.clone())
                .to(Mailbox {
                    name: receiver_name,
                    email: receiver_email,
                })
                .header(ContentType::TEXT_PLAIN)
                .subject(subject.as_ref())
                .body(body)?,
        )?)
    }
}
