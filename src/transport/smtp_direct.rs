use lettre::Transport;
use lettre::message::{Message, header::ContentType};
use lettre::{
  SmtpTransport,
  transport::smtp::client::{Tls, TlsParameters},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email::get_mx_records;
use crate::log;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpDirect {
  recipient_email: String,
  sender: String,
  hostname: String,
}

impl SmtpDirect {
  pub fn new(recipient_email: String, sender: String, hostname: String) -> Self {
    Self {
      recipient_email,
      sender,
      hostname,
    }
  }
}

impl super::TransportService for SmtpDirect {
  fn send(&self, title: String, body: String) -> anyhow::Result<()> {
    let mxs = get_mx_records(self.recipient_email.as_str())?;
    if mxs.is_empty() {
      return Err(anyhow::anyhow!(
        "No MX records found for domain of recipient email: {}",
        self.recipient_email
      ));
    }
    for mx in &mxs {
      let tls = TlsParameters::new(mx.exchange.clone().to_string())?;
      let mailer = SmtpTransport::builder_dangerous(mx.exchange.clone())
        .port(25)
        .tls(Tls::Opportunistic(tls))
        .build();
      let message_id = format!("<{}@{}>", Uuid::new_v4(), self.hostname);
      let m = Message::builder()
        .from(self.sender.parse()?)
        .to(self.recipient_email.parse()?)
        .date_now()
        .message_id(Some(message_id))
        .header(ContentType::TEXT_PLAIN)
        .subject(title.clone())
        .body(body.clone())?;
      let res = mailer.send(&m);
      if res.is_ok() {
        return Ok(());
      }
      log::debug(&format!(
        "Failed to send email to {} via MX record {}: {:?}",
        self.recipient_email,
        mx.exchange,
        res.err()
      ));
    }

    return Err(anyhow::anyhow!(
      "Failed to send email to {} via all MX records: {:?}",
      self.recipient_email,
      mxs
        .iter()
        .map(|mx| mx.exchange.to_string())
        .collect::<Vec<_>>()
    ));
  }
}
