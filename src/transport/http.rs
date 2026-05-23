use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ureq;

use crate::{transport::TransportService, utils::retry_with_backoff};

#[derive(Debug, Clone, Deserialize, Serialize)]
enum Method {
  POST,
  PUT,
  PATCH,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Http {
  method: Method,
  uri: String,
  headers: HashMap<String, String>,
  query_params: HashMap<String, String>,
  json_body: Option<bool>,
}

impl Http {
  pub fn new() -> Self {
    Http {}
  }
}

impl TransportService for Http {
  fn send(&self, title: String, body: String) -> anyhow::Result<()> {
    let response = retry_with_backoff(|| , 3, 100)
    Ok(())
  }
}
