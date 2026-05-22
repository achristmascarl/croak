use clap::Parser;

use crate::cli::Cli;

mod cli;
mod config;
mod email;
mod log;
mod process;
mod transport;
mod utils;

fn main() -> anyhow::Result<()> {
  init_panic_hook();
  let args = Cli::parse();
  if args.command == Some(cli::CliCommand::Edit) {
    return config::edit_config_file();
  }
  let cfg = config::Config::new()?;
  process::run(args.target_args, cfg)?;
  Ok(())
}

fn init_panic_hook() {
  std::panic::set_hook(Box::new(|panic_info| {
    let payload = panic_info.payload();
    let message = if let Some(s) = payload.downcast_ref::<&str>() {
      *s
    } else if let Some(s) = payload.downcast_ref::<String>() {
      s.as_str()
    } else {
      "Unknown panic payload"
    };
    log::error(&format!("Panic occurred: {}", message));
  }));
}
