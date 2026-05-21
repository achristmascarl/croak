use clap::Parser;

use crate::utils::version;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Option<CliCommand>,

  #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
  pub target_args: Vec<String>,
}

#[derive(clap::Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCommand {
  Edit,
}
