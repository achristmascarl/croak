use clap::Parser;

use crate::cli::Cli;

mod cli;
mod config;
mod log;
mod process;
mod transport;
mod utils;

fn main() -> anyhow::Result<()> {
  init_panic_hook();
  let args = Cli::parse();
  if args.command == Some(cli::CliCommand::Edit) {
    return config::edit_config_file();
  } else if let Some(cli::CliCommand::Configure { transport }) = args.command {
    return transport::configure_transport(transport);
  }
  let cfg_result = config::Config::new();
  let Ok(cfg) = cfg_result else {
    log::error(&format!(
      "Failed to load configuration: {:?}",
      cfg_result.as_ref().err()
    ));
    std::process::exit(1);
  };
  let run_result = process::run(args.target_args, cfg);
  if let Err(e) = run_result {
    log::error(&format!("Error running command: {:?}", e));
    std::process::exit(1);
  }
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
