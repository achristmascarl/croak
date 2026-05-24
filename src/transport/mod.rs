use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::fmt;

use http::Http;

use crate::{cli::TransportKind, log};

pub mod http;

#[enum_dispatch]
pub trait TransportService {
  /// Get the name of the transport.
  fn name(&self) -> &str;

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
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Transport {
  Http,
}

impl fmt::Display for Transport {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Transport::Http(http) => write!(f, "[HTTP] {}", http.name()),
    }
  }
}

pub fn configure_transport(transport_kind: TransportKind) -> anyhow::Result<()> {
  match transport_kind {
    TransportKind::Http => http::configure()?,
  };
  Ok(())
}

pub fn notify_first(
  transports: &[Transport],
  title: String,
  body: String,
  bail_on_fail: bool,
) -> anyhow::Result<()> {
  let mut sent = false;
  for transport in transports {
    let transport_name = transport.name().to_string();
    if let Err(e) = transport.send(title.clone(), body.clone()) {
      log::error(&format!(
        "Failed to send notification via transport '{}': {:?}",
        transport_name, e
      ));
    } else {
      sent = true;
      break;
    }
  }
  if !sent && bail_on_fail {
    anyhow::bail!(
      "Failed to send notification via all {} transports",
      transports.len()
    );
  }
  Ok(())
}
