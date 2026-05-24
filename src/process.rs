use crate::{config::Config, log, transport::TransportService};

use std::{
  process::{Child, ExitStatus},
  sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
  },
  thread,
  time::Duration,
};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
static CTRL_C_HANDLER_INSTALL: OnceLock<Result<(), String>> = OnceLock::new();

pub fn run(target: Vec<String>, cfg: Config) -> anyhow::Result<ExitStatus> {
  if target.is_empty() {
    anyhow::bail!("No target command provided. Nothing to run.");
  }
  let transports = cfg.transports;
  if transports.is_empty() {
    anyhow::bail!("No transports configured. Notifications cannot be sent.");
  }
  let mut cmd = std::process::Command::new(&target[0]);
  let cmd_name = cmd.get_program().to_string_lossy().to_string();
  for arg in &target[1..] {
    cmd.arg(arg);
  }
  INTERRUPTED.store(false, Ordering::SeqCst);
  install_ctrlc_handler()?;
  log::info(&format!("Running command: {}", cmd_name));
  let child = cmd
    .stdout(std::process::Stdio::inherit())
    .stderr(std::process::Stdio::inherit())
    .spawn()?;
  let mut child = ChildGuard::new(child, cmd_name.clone());
  let status = child.wait()?;
  log::info(&format!(
    "Command {} exited with status: {}",
    cmd_name, status
  ));
  let hostname = cfg
    .settings
    .override_hostname
    .unwrap_or(hostname::get().map_or("croak".into(), |h| h.to_string_lossy().to_string()));
  for transport in transports {
    let transport_name = transport.name().to_string();
    let title = format!(
      "[{}] Command '{}' exited with status: {}",
      hostname, cmd_name, status
    );
    let body = format!(
      "The command '{}' was executed and exited with status: {}.",
      cmd_name, status
    );
    if let Err(e) = transport.send(title, body) {
      log::error(&format!(
        "Failed to send notification via transport '{}': {:?}",
        transport_name, e
      ));
    }
  }
  Ok(status)
}

fn install_ctrlc_handler() -> anyhow::Result<()> {
  let result = CTRL_C_HANDLER_INSTALL.get_or_init(|| {
    ctrlc::set_handler(|| {
      INTERRUPTED.store(true, Ordering::SeqCst);
    })
    .map_err(|e| e.to_string())
  });

  if let Err(e) = result {
    anyhow::bail!("Failed to install Ctrl-C handler: {}", e);
  }

  Ok(())
}

struct ChildGuard {
  child: Option<Child>,
  cmd_name: String,
}

impl ChildGuard {
  fn new(child: Child, cmd_name: String) -> Self {
    Self {
      child: Some(child),
      cmd_name,
    }
  }

  fn wait(&mut self) -> anyhow::Result<ExitStatus> {
    loop {
      let child = self
        .child
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Child process was already reaped"))?;
      if let Some(status) = child.try_wait()? {
        self.child.take();
        return Ok(status);
      }

      if INTERRUPTED.load(Ordering::SeqCst) {
        anyhow::bail!("Interrupted while running command '{}'", self.cmd_name);
      }

      thread::sleep(Duration::from_millis(100));
    }
  }
}

impl Drop for ChildGuard {
  fn drop(&mut self) {
    let Some(mut child) = self.child.take() else {
      return;
    };

    match child.try_wait() {
      Ok(Some(_)) => {},
      Ok(None) => {
        log::info(&format!(
          "Killing command '{}' because croak is exiting",
          self.cmd_name
        ));
        if let Err(e) = child.kill() {
          log::error(&format!(
            "Failed to kill command '{}': {:?}",
            self.cmd_name, e
          ));
        }
        if let Err(e) = child.wait() {
          log::error(&format!(
            "Failed to reap command '{}': {:?}",
            self.cmd_name, e
          ));
        }
      },
      Err(e) => {
        log::error(&format!(
          "Failed to inspect command '{}' before cleanup: {:?}",
          self.cmd_name, e
        ));
      },
    }
  }
}
