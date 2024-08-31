use anyhow::anyhow;
use email_address::EmailAddress;
use lettre::transport::smtp::response::Response as LetterResponse;
use lettre::{
    self,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

use crate::{config::Config, AppError};

pub struct Mailer {
    sender: Mailbox,
    mailer: SmtpTransport,
}

impl Mailer {
    pub fn from(env: &Config) -> Result<Self, AppError> {
        let from_email = env
            .smtp_from_email
            .as_ref()
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
            .filter(|a| EmailAddress::is_valid(a))
            .ok_or(AppError::from(anyhow!(
                "invalid smtp from email, contact the server owner"
            )))?;
        let from_name = env
            .smtp_from_name
            .as_ref()
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
            .ok_or(AppError::from(anyhow!(
                "invalid smtp from name, contact the server owner"
            )))?;
        let sender: Mailbox = format!("{} <{}>", from_name, from_email)
            .parse()
            .map_err(|_| {
                AppError::from(anyhow!("invalid sender info, contact the server owner"))
            })?;

        let host = env
            .smtp_host
            .as_ref()
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
            .ok_or(AppError::from(anyhow!(
                "invalid smtp host, contact the server owner"
            )))?;
        let smtp_username = env
            .smtp_username
            .as_ref()
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
            .ok_or(AppError::from(anyhow!(
                "invalid smtp username, contact the server owner"
            )))?;
        let smtp_password = env
            .smtp_password
            .as_ref()
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
            .ok_or(AppError::from(anyhow!(
                "invalid smtp password, contact the server owner"
            )))?;

        let mailer = SmtpTransport::relay(host)
            .map_err(|e| AppError::from(anyhow!("can't create mailer: {}", e)))?
            .credentials(Credentials::new(
                smtp_username.to_string(),
                smtp_password.to_string(),
            ))
            .build();

        Ok(Self { mailer, sender })
    }

    pub fn send(
        &self,
        receiver_name: impl AsRef<str>,
        receiver_email: impl AsRef<str>,
        subject: impl AsRef<str>,
        content: impl AsRef<str>,
    ) -> Result<LetterResponse, AppError> {
        let to: Mailbox = format!("{} <{}>", receiver_name.as_ref(), receiver_email.as_ref())
            .parse()
            .map_err(|_| {
                AppError::from(anyhow!(
                    "invalid receiver info: {}",
                    receiver_email.as_ref()
                ))
            })?;

        let email = Message::builder()
            .from(self.sender.clone())
            .to(to)
            .header(ContentType::TEXT_PLAIN)
            .subject(subject.as_ref())
            .body(content.as_ref().to_string())
            .map_err(|e| AppError::from(anyhow!("can't build email: {}", e)))?;

        self.mailer
            .send(&email)
            .map_err(|e| AppError::from(anyhow!("can't send email: {}", e)))
    }
}
