use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use http::Http;

pub mod http;

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
  Http,
}

pub fn init_transports(cfg: config::Config) -> anyhow::Result<Vec<Transport>> {
  let mut transports: Vec<Transport> = Vec::new();

  Ok(transports)
}
