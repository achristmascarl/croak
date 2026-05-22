use crate::{
  config::Config,
  log,
  transport::{self, TransportService},
};

pub fn run(target: Vec<String>, cfg: Config) -> anyhow::Result<()> {
  if target.is_empty() {
    anyhow::bail!("No target command provided. Nothing to run.");
  }
  let transports = transport::init_transports(cfg)?;
  let mut cmd = std::process::Command::new(&target[0]);
  let cmd_name = cmd.get_program().to_string_lossy().to_string();
  for arg in &target[1..] {
    cmd.arg(arg);
  }
  log::info(&format!("Running command: {}", cmd_name));
  let status = cmd.stdout(std::process::Stdio::inherit()) // Redirects subprocess stdout to parent stdout
      .stderr(std::process::Stdio::inherit()) // Redirects subprocess stderr to parent stderr
      .status()?;
  log::info(&format!(
    "Command {} exited with status: {}",
    cmd_name, status
  ));
  if transports.is_empty() {
    log::error("No transports configured. No notifications will be sent.");
  }
  for transport in transports {
    let title = format!("Command '{}' exited with status: {}", cmd_name, status);
    let body = format!(
      "The command '{}' was executed and exited with status: {}.",
      cmd_name, status
    );
    if let Err(e) = transport.send(title, body) {
      log::error(&format!(
        "Failed to send notification via transport: {:?}",
        e
      ));
    }
  }
  Ok(())
}
