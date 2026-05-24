use std::collections::HashSet;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

use serde::Deserialize;

use crate::{
  log,
  transport::{Transport, TransportService},
};

const CONFIG: &str = include_str!("../.config/config.toml");
const CONFIG_FILE_CANDIDATES: [(&str, config::FileFormat); 5] = [
  ("config.json5", config::FileFormat::Json5),
  ("config.json", config::FileFormat::Json),
  ("config.yaml", config::FileFormat::Yaml),
  ("config.toml", config::FileFormat::Toml),
  ("config.ini", config::FileFormat::Ini),
];
const PREFERRED_CONFIG_FILENAME: &str = "config.toml";

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Folders {
  #[serde(default)]
  pub _data_dir: PathBuf,
  #[serde(default)]
  pub _config_dir: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Settings {
  pub notify_on_start: Option<bool>,
  pub override_hostname: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
  #[serde(default, flatten)]
  pub _folders: Folders,
  #[serde(default)]
  pub settings: Settings,
  #[serde(default)]
  pub transports: Vec<Transport>,
}

impl Config {
  pub fn new() -> anyhow::Result<Self> {
    let default_config: Config = toml::from_str(CONFIG).unwrap();
    let data_dir = crate::utils::get_data_dir();
    let config_dir = crate::utils::get_config_dir();
    let mut builder = config::Config::builder()
      .set_default("_data_dir", data_dir.to_str().unwrap())?
      .set_default("_config_dir", config_dir.to_str().unwrap())?;

    let mut found_config = false;
    for (file, format) in &CONFIG_FILE_CANDIDATES {
      builder = builder.add_source(
        config::File::from(config_dir.join(file))
          .format(*format)
          .required(false),
      );
      if config_dir.join(file).exists() {
        found_config = true
      }
    }
    if !found_config {
      log::warn("No configuration file found. Program may not behave as expected");
    }

    let mut cfg: Self = builder.build()?.try_deserialize()?;
    cfg.validate()?;
    if cfg.settings.notify_on_start.is_none() {
      cfg.settings.notify_on_start = default_config.settings.notify_on_start;
    }

    Ok(cfg)
  }

  fn validate(&mut self) -> anyhow::Result<()> {
    if let Some(ref hostname) = self.settings.override_hostname
      && hostname.trim().is_empty()
    {
      self.settings.override_hostname = None;
    }

    let mut transport_name_set: HashSet<String> = HashSet::new();
    for transport in &self.transports {
      let name = transport.name().to_string();
      if transport_name_set.contains(&name) {
        anyhow::bail!(
          "Duplicate transport name found: '{}'. Each transport must have a unique name.",
          name
        );
      }
      transport_name_set.insert(name);
    }

    Ok(())
  }

  pub fn list_transports(&self) -> anyhow::Result<()> {
    if self.transports.is_empty() {
      println!("No transports configured.");
    } else {
      println!("Configured transports:");
      for transport in &self.transports {
        println!("- {}", transport);
      }
    }
    Ok(())
  }
}

pub fn edit_config_file() -> anyhow::Result<()> {
  let config_path = existing_config_path().unwrap_or_else(preferred_config_path);
  ensure_config_file(&config_path)?;
  open_editor(&config_path)
}

fn preferred_config_path() -> PathBuf {
  crate::utils::get_config_dir().join(PREFERRED_CONFIG_FILENAME)
}

fn existing_config_path() -> Option<PathBuf> {
  let config_dir = crate::utils::get_config_dir();
  CONFIG_FILE_CANDIDATES
    .iter()
    .map(|(name, _)| config_dir.join(name))
    .find(|path| path.exists())
}

fn default_config_contents() -> &'static str {
  CONFIG
}

fn ensure_config_file(path: &Path) -> anyhow::Result<()> {
  if path.exists() {
    let contents = fs::read_to_string(path)?;
    if !contents.trim().is_empty() {
      return Ok(());
    }
  }

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  write_default_config(path)?;
  Ok(())
}

fn write_default_config(path: &Path) -> anyhow::Result<()> {
  let mut options = fs::OpenOptions::new();
  options.write(true).create(true).truncate(true);
  #[cfg(unix)]
  options.mode(0o600);

  let mut file = options.open(path)?;
  file.write_all(default_config_contents().as_bytes())?;
  set_config_file_permissions(path)?;
  Ok(())
}

#[cfg(unix)]
fn set_config_file_permissions(path: &Path) -> anyhow::Result<()> {
  fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
  Ok(())
}

#[cfg(not(unix))]
fn set_config_file_permissions(_path: &Path) -> anyhow::Result<()> {
  Ok(())
}

fn open_editor(path: &Path) -> anyhow::Result<()> {
  let editor = std::env::var("VISUAL")
    .ok()
    .filter(|value| !value.trim().is_empty())
    .or_else(|| {
      std::env::var("EDITOR")
        .ok()
        .filter(|value| !value.trim().is_empty())
    })
    .unwrap_or_else(|| "vi".to_string());

  let mut parts = editor.split_whitespace();
  let program = parts
    .next()
    .ok_or_else(|| anyhow::anyhow!("Could not parse editor command"))?;
  let status = Command::new(program).args(parts).arg(path).status()?;
  if !status.success() {
    anyhow::bail!("Editor exited with status code {:?}", status.code());
  }
  Ok(())
}

pub fn append_to_config_file(contents: &str) -> anyhow::Result<()> {
  let config_path = existing_config_path().unwrap_or_else(preferred_config_path);
  ensure_config_file(&config_path)?;
  let mut file = fs::OpenOptions::new().append(true).open(&config_path)?;

  writeln!(file)?;
  writeln!(file, "{contents}")?;
  writeln!(file)?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::transport::TransportService;
  #[cfg(unix)]
  use std::os::unix::fs::PermissionsExt;

  #[test]
  fn parses_tagged_transports() {
    let cfg: Config = toml::from_str(
      r#"
        [[transports]]
        type = "Http"
        name = "notify"
        method = "POST"
        uri = "https://example.com/notify"
        headers = { Authorization = "Bearer token" }
        query_params = { source = "croak" }
        json_body = true
      "#,
    )
    .unwrap();

    assert_eq!(cfg.transports.len(), 1);
    assert_eq!(cfg.transports.get(0).unwrap().name(), "notify");
  }

  #[test]
  fn transports_default_to_empty() {
    let cfg: Config = toml::from_str("").unwrap();

    assert!(cfg.transports.is_empty());
  }

  #[test]
  fn validates_config_after_loading() {
    let mut cfg: Config = toml::from_str(
      r#"
        [settings]
        override_hostname = "   "

        [[transports]]
        type = "Http"
        name = "notify"
        method = "POST"
        uri = "https://example.com/notify"

        [[transports]]
        type = "Http"
        name = "notify"
        method = "PUT"
        uri = "https://example.com/notify"
      "#,
    )
    .unwrap();

    let err = cfg.validate().unwrap_err();

    assert!(cfg.settings.override_hostname.is_none());
    assert!(err.to_string().contains("Duplicate transport name"));
  }

  #[cfg(unix)]
  #[test]
  fn creates_config_file_with_owner_only_permissions() {
    let test_dir = std::env::temp_dir().join(format!("croak-config-test-{}", uuid::Uuid::new_v4()));
    let config_path = test_dir.join("config.toml");

    ensure_config_file(&config_path).unwrap();

    let mode = fs::metadata(&config_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);

    fs::remove_dir_all(test_dir).unwrap();
  }
}
