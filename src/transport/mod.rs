use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

pub mod smtp_direct;
pub mod smtp_relay;

use smtp_direct::SmtpDirect;
use smtp_relay::SmtpRelay;

use crate::{config, utils::prompt_for_input};

#[enum_dispatch]
pub trait TransportService {
  /// Send data to the destination. The implementor
  /// is responsible for defining the destination or offering
  /// a way to set it.
  fn send(&self, title: String, body: String) -> anyhow::Result<()>;
}

/// A Transport is a way to send data to a destination.
/// This could be via an email, an HTTP request, an API, etc.
/// There should be one Transport per each destination; for example,
/// if you want to send data to two different email addresses, you
/// should have two Transports, one for each email address.
#[enum_dispatch(TransportService)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Transport {
  SmtpDirect,
  SmtpRelay,
}

pub fn init_transports(cfg: config::Config) -> anyhow::Result<Vec<Transport>> {
  let mut transports: Vec<Transport> = Vec::new();

  // Always include direct SMTP transport as a last resort fallback
  let default_fallback_username_path = cfg
    .folders
    ._config_dir
    .join("default_fallback_smtp_username.txt");
  let fallback_username =
    cfg
      .settings
      .fallback_smtp_username
      .unwrap_or(config::ensure_default_fallback_smtp_username(
        &default_fallback_username_path,
      )?);
  let fallback_hostname = format!(
    "{}.local",
    cfg
      .settings
      .fallback_smtp_hostname
      .unwrap_or(hostname::get().map_or("croak".to_string(), |h| h.to_string_lossy().to_string()))
  );
  let sender = format!("{}@{}", fallback_username, fallback_hostname);
  let recipient_email = cfg
    .settings
    .fallback_recipient_email
    .unwrap_or(prompt_for_input("Email to receive notifications at: ")?);
  transports.push(Transport::SmtpDirect(SmtpDirect::new(
    recipient_email,
    sender,
    fallback_hostname,
  )));

  Ok(transports)
}
