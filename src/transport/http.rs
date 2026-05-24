use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};
use ureq::{self, http::StatusCode};

use crate::{
  config,
  transport::{Transport, TransportService},
  utils::{self, retry_with_backoff},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Method {
  POST,
  PUT,
  PATCH,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Http {
  name: String,
  method: Method,
  uri: String,
  headers: Option<HashMap<String, String>>,
  query_params: Option<HashMap<String, String>>,
  json_body: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct HttpJsonPayload {
  title: String,
  body: String,
}

impl TransportService for Http {
  fn name(&self) -> &str {
    &self.name
  }

  fn send(&self, title: String, body: String) -> anyhow::Result<()> {
    let _ = retry_with_backoff(
      || {
        let agent: ureq::Agent = ureq::Agent::config_builder()
          .timeout_global(Some(REQUEST_TIMEOUT))
          .build()
          .into();
        let mut request = match self.method {
          Method::POST => agent.post(&self.uri),
          Method::PUT => agent.put(&self.uri),
          Method::PATCH => agent.patch(&self.uri),
        };
        if let Some(query_params) = &self.query_params {
          for (key, value) in query_params {
            request = request.query(key, value);
          }
        }
        if let Some(headers) = &self.headers {
          for (key, value) in headers {
            request = request.header(key.clone(), value.clone());
          }
        }
        let response = if self.json_body.unwrap_or(false) {
          let payload = HttpJsonPayload {
            title: title.clone(),
            body: body.clone(),
          };
          request.send_json(&payload)?
        } else {
          let payload = format!(
            "{}\n{}\n\n{}",
            title,
            "-".repeat(title.len().min(100)),
            body
          );
          request.send(payload)?
        };
        if response.status() != StatusCode::OK {
          anyhow::bail!("Received non-success status code: {}", response.status());
        }
        Ok(response.status())
      },
      3,
      100,
    )?;
    Ok(())
  }
}

pub fn configure() -> anyhow::Result<()> {
  let name = utils::prompt_for_input("Enter a unique name for this HTTP transport: ")?
    .trim()
    .to_string();
  let uri = utils::prompt_for_input("Enter the URI to send notifications to: ")?
    .trim()
    .to_string();
  let method_str = utils::prompt_for_input_with_validation(
    "Enter the HTTP method to use (POST, PUT, PATCH): ",
    |s| matches!(s.to_uppercase().as_str(), "POST" | "PUT" | "PATCH"),
    "Invalid HTTP method. Please enter POST, PUT, or PATCH.",
  )?;
  let method = match method_str.to_uppercase().trim() {
    "POST" => Method::POST,
    "PUT" => Method::PUT,
    "PATCH" => Method::PATCH,
    _ => anyhow::bail!("Invalid HTTP method: {}", method_str),
  };
  let json_body_str = utils::prompt_for_input("Should the body be sent as JSON? [yes/no] (no): ")?;
  let json_body = matches!(json_body_str.to_lowercase().trim(), "yes" | "y");
  let mut headers: Option<HashMap<String, String>> = None;
  let should_add_headers_str =
    utils::prompt_for_input("Do you want to add custom headers? [yes/no] (no): ")?;
  let should_add_headers = matches!(should_add_headers_str.to_lowercase().trim(), "yes" | "y");
  if should_add_headers {
    let mut added_headers = HashMap::new();
    loop {
      let header_input = utils::prompt_for_input(
        "Enter a header in the format 'Key: Value' (or leave blank to finish): ",
      )?;
      if header_input.trim().is_empty() {
        headers = Some(added_headers);
        break;
      }
      if let Some((key, value)) = header_input.split_once(':') {
        added_headers.insert(key.trim().to_string(), value.trim().to_string());
      } else {
        println!("Invalid header format. Please enter in the format 'Key: Value'.");
      }
    }
  }
  let mut query_params: Option<HashMap<String, String>> = None;
  let should_add_query_params_str =
    utils::prompt_for_input("Do you want to add custom query parameters? [yes/no] (no): ")?;
  let should_add_query_params = matches!(
    should_add_query_params_str.to_lowercase().trim(),
    "yes" | "y"
  );
  if should_add_query_params {
    let mut added_query_params = HashMap::new();
    loop {
      let query_param_input = utils::prompt_for_input(
        "Enter a query parameter in the format 'Key=Value' (or leave blank to finish): ",
      )?;
      if query_param_input.trim().is_empty() {
        query_params = Some(added_query_params);
        break;
      }
      if let Some((key, value)) = query_param_input.split_once('=') {
        added_query_params.insert(key.trim().to_string(), value.trim().to_string());
      } else {
        println!("Invalid query parameter format. Please enter in the format 'Key=Value'.");
      }
    }
  }
  let http_transport = Transport::Http(Http {
    name,
    method,
    uri,
    headers,
    query_params,
    json_body: Some(json_body),
  });
  let config_string = format!("[[transports]]\n{}", toml::to_string(&http_transport)?);
  config::append_to_config_file(config_string.as_str())?;
  println!("HTTP transport configured successfully!");
  Ok(())
}
