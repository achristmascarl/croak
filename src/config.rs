use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

use serde::Deserialize;

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

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
  #[serde(default, flatten)]
  pub folders: Folders,
}

impl Config {
  pub fn new() -> Result<Self, config::ConfigError> {
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
      eprintln!("No configuration file found. Program may not behave as expected");
    }

    let mut cfg: Self = builder.build()?.try_deserialize()?;

    Ok(cfg)
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
    return Ok(());
  }

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::write(path, default_config_contents())?;
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
