mod favorite_bookmark;
mod get_whoami;
mod post_modify;
mod post_sensitive;
mod put_progress;

pub use favorite_bookmark::*;
pub use get_whoami::*;
pub use post_modify::*;
pub use post_sensitive::*;
pub use put_progress::*;

use anyhow::{Result, anyhow};
use email_address::EmailAddress;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{IntoBody, Mailbox, header::ContentType},
    transport::smtp::{authentication::Credentials, response::Response as LetterResponse},
};

use crate::utils::config::Config;

pub struct Mailer {
    sender: Mailbox,
    mailer: SmtpTransport,
}

impl Mailer {
    pub fn from(env: &Config) -> Result<Self> {
        let smtp = env
            .smtp
            .as_ref()
            .ok_or_else(|| anyhow!("SMTP was not fully configured, contact the server owner"))?;

        if !EmailAddress::is_valid(&smtp.from_email) {
            return Err(anyhow!("Invalid SMTP from email, contact the server owner"));
        }

        let sender: Mailbox = format!("{} <{}>", smtp.from_name, smtp.from_email)
            .parse()
            .map_err(|_| anyhow!("invalid sender info, contact the server owner"))?;

        let mailer = SmtpTransport::relay(&smtp.host)
            .map_err(|e| anyhow!("can't create mailer: {}", e))?
            .credentials(Credentials::new(
                smtp.username.to_string(),
                smtp.password.to_string(),
            ))
            .build();

        #[allow(clippy::inconsistent_struct_constructor)]
        Ok(Self { mailer, sender })
    }

    pub fn send(
        &self,
        receiver_name: impl AsRef<str>,
        receiver_email: impl AsRef<str>,
        subject: impl AsRef<str>,
        body: impl IntoBody,
    ) -> Result<LetterResponse> {
        let to: Mailbox = format!("{} <{}>", receiver_name.as_ref(), receiver_email.as_ref())
            .parse()
            .map_err(|_| anyhow!("invalid receiver info: {}", receiver_email.as_ref()))?;

        let email = Message::builder()
            .from(self.sender.clone())
            .to(to)
            .header(ContentType::TEXT_PLAIN)
            .subject(subject.as_ref())
            .body(body)
            .map_err(|e| anyhow!("can't build email: {e}"))?;

        self.mailer
            .send(&email)
            .map_err(|e| anyhow!("can't send email: {e}"))
    }
}
