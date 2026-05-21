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
  let args = Cli::parse();
  if args.command == Some(cli::CliCommand::Edit) {
    return config::edit_config_file();
  }
  let cfg = config::Config::new()?;
  process::run(args.target_args, cfg)?;
  Ok(())
}
