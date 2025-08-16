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
        let from_email = env
            .smtp_from_email
            .as_ref()
            .filter(|a| EmailAddress::is_valid(a))
            .ok_or_else(|| anyhow!("invalid smtp from email, contact the server owner"))?;
        let from_name = env
            .smtp_from_name
            .as_ref()
            .ok_or_else(|| anyhow!("invalid smtp from name, contact the server owner"))?;
        let sender: Mailbox = format!("{from_name} <{from_email}>")
            .parse()
            .map_err(|_| anyhow!("invalid sender info, contact the server owner"))?;

        let host = env
            .smtp_host
            .as_ref()
            .ok_or_else(|| anyhow!("invalid smtp host, contact the server owner"))?;
        let smtp_username = env
            .smtp_username
            .as_ref()
            .ok_or_else(|| anyhow!("invalid smtp username, contact the server owner"))?;
        let smtp_password = env
            .smtp_password
            .as_ref()
            .ok_or_else(|| anyhow!("invalid smtp password, contact the server owner"))?;

        let mailer = SmtpTransport::relay(host)
            .map_err(|e| anyhow!("can't create mailer: {}", e))?
            .credentials(Credentials::new(
                smtp_username.to_string(),
                smtp_password.to_string(),
            ))
            .build();

        #[allow(clippy::inconsistent_struct_constructor)]
        Ok(Self { mailer, sender })
    }

    pub fn send(
        &self,
        receiver_name: &str,
        receiver_email: &str,
        subject: impl Into<String>,
        body: impl IntoBody,
    ) -> Result<LetterResponse> {
        let to: Mailbox = format!("{receiver_name} <{receiver_email}>")
            .parse()
            .map_err(|_| anyhow!("invalid receiver info: {receiver_email}"))?;

        let email = Message::builder()
            .from(self.sender.clone())
            .to(to)
            .header(ContentType::TEXT_PLAIN)
            .subject(subject)
            .body(body)
            .map_err(|e| anyhow!("can't build email: {e}"))?;

        self.mailer
            .send(&email)
            .map_err(|e| anyhow!("can't send email: {e}"))
    }
}
