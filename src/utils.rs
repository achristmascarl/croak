use lazy_static::lazy_static;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

const VERSION_MESSAGE: &str = concat!(
  env!("CARGO_PKG_VERSION"),
  "-",
  env!("VERGEN_GIT_DESCRIBE"),
  " (",
  env!("VERGEN_BUILD_DATE"),
  ")"
);

lazy_static! {
  pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
  pub static ref DATA_FOLDER: Option<PathBuf> =
    std::env::var(format!("{}_DATA", PROJECT_NAME.clone()))
      .ok()
      .map(PathBuf::from);
  pub static ref CONFIG_FOLDER: Option<PathBuf> =
    std::env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
      .ok()
      .map(PathBuf::from);
  pub static ref EXPORT_FOLDER: Option<PathBuf> =
    std::env::var(format!("{}_EXPORT", PROJECT_NAME.clone()))
      .ok()
      .map(PathBuf::from);
  pub static ref FAVORITES_FOLDER: Option<PathBuf> =
    std::env::var(format!("{}_FAVORITES", PROJECT_NAME.clone()))
      .ok()
      .map(PathBuf::from);
  pub static ref LOG_ENV: String = format!("{}_LOGLEVEL", PROJECT_NAME.clone());
  pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

pub fn get_data_dir() -> PathBuf {
  if let Some(s) = DATA_FOLDER.clone() {
    return s;
  } else if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
    return PathBuf::from(xdg_data_home).join(env!("CARGO_PKG_NAME"));
  } else if let Some(home_dir) = dirs::home_dir() {
    return home_dir.join(".local/share").join(env!("CARGO_PKG_NAME"));
  } else {
    return PathBuf::from(".").join(".data");
  }
}

pub fn get_config_dir() -> PathBuf {
  if let Some(s) = CONFIG_FOLDER.clone() {
    return s;
  } else if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME") {
    return PathBuf::from(xdg_config_home).join(env!("CARGO_PKG_NAME"));
  } else if let Some(home_dir) = dirs::home_dir() {
    return home_dir.join(".config").join(env!("CARGO_PKG_NAME"));
  } else {
    return PathBuf::from(".").join(".config");
  }
}

pub fn version() -> String {
  let author = clap::crate_authors!();

  let config_dir_path = get_config_dir().display().to_string();
  let data_dir_path = get_data_dir().display().to_string();

  format!(
    "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
  )
}

pub fn prompt_for_input(prompt: &str) -> anyhow::Result<String> {
  let mut response = String::new();
  print!("{prompt}");
  io::stdout().flush()?;
  io::stdin().read_line(&mut response)?;
  Ok(response.trim().to_string())
}

pub fn retry_with_backoff<T, E, F>(
  mut f: F,
  max_retries: u32,
  base_backoff_ms: u64,
) -> anyhow::Result<T>
where
  E: std::fmt::Display,
  F: FnMut() -> anyhow::Result<T>,
{
  for attempt in 0..=max_retries {
    match f() {
      Ok(value) => return Ok(value),
      Err(err) if attempt == max_retries => return Err(anyhow::anyhow!(err.to_string())),
      Err(_) => thread::sleep(Duration::from_millis(base_backoff_ms * 2_u64.pow(attempt))),
    }
  }
  unreachable!("retry loop always returns")
}
