use clap::Parser;

use crate::cli::Cli;
use crate::transport::TransportService;

mod cli;
mod config;
mod email;
mod process;
mod transport;
mod utils;

fn main() -> anyhow::Result<()> {
  let args = Cli::parse();
  if args.command == Some(cli::CliCommand::Edit) {
    return config::edit_config_file();
  }
  let cfg = config::Config::new()?;
  let smtp =
    transport::smtp_direct::SmtpDirect::new("carl@atlas.net".into(), "test@test.com".into());
  smtp.send("Hello world".into(), "hello there".into())?;
  process::run(args.target_args)?;
  Ok(())
}
