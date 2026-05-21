use hostname;
use lettre::Transport;
use lettre::message::{Message, header::ContentType};
use lettre::{
  SmtpTransport,
  transport::smtp::client::{Tls, TlsParameters},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::email::get_mx_record;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpDirect {
  recipient_email: String,
  sender: String,
}

impl SmtpDirect {
  pub fn new(recipient_email: String, sender: String) -> Self {
    Self {
      recipient_email,
      sender,
    }
  }
}

impl super::TransportService for SmtpDirect {
  fn send(&self, title: String, body: String) -> anyhow::Result<()> {
    let mx = get_mx_record(self.recipient_email.as_str())?;
    let tls = TlsParameters::new(mx.exchange.to_string())?;
    let mailer = SmtpTransport::builder_dangerous(mx.exchange)
      .port(25)
      .tls(Tls::Opportunistic(tls))
      .build();
    let message_id = format!(
      "<{}@{}>",
      Uuid::new_v4(),
      hostname::get()
        .unwrap_or("croak.local".into())
        .to_string_lossy()
    );
    let m = Message::builder()
      .from(self.sender.parse()?)
      .to(self.recipient_email.parse()?)
      .date_now()
      .message_id(Some(message_id))
      .header(ContentType::TEXT_PLAIN)
      .subject(title)
      .body(body)?;
    let res = mailer.send(&m)?;
    println!("Email sent: {:?}", res);

    Ok(())
  }
}
