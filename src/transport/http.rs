use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ureq;

use crate::{transport::TransportService, utils::retry_with_backoff};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Method {
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
  pub fn new(
    method: Method,
    uri: String,
    headers: HashMap<String, String>,
    query_params: HashMap<String, String>,
    json_body: Option<bool>,
  ) -> Self {
    Http {
      method,
      uri,
      headers,
      query_params,
      json_body,
    }
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct HttpJsonPayload {
  title: String,
  body: String,
}

impl TransportService for Http {
  fn send(&self, title: String, body: String) -> anyhow::Result<()> {
    let _ = retry_with_backoff(
      || {
        let mut request = match self.method {
          Method::POST => ureq::post(&self.uri),
          Method::PUT => ureq::put(&self.uri),
          Method::PATCH => ureq::patch(&self.uri),
        };
        for (key, value) in &self.headers {
          request = request.header(key.clone(), value.clone());
        }
        if self.json_body.unwrap_or(false) {
          let payload = HttpJsonPayload {
            title: title.clone(),
            body: body.clone(),
          };
          Ok(request.send_json(&payload)?.body_mut().read_to_string())
        } else {
          let payload = format!("{}\n{}\n\n{}", title, title.len().min(100), body);
          Ok(request.send(payload)?.body_mut().read_to_string())
        }
      },
      3,
      100,
    )?;
    Ok(())
  }
}
