use clap::{Parser, ValueEnum};

use crate::utils::version;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Option<CliCommand>,

  #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
  pub target_args: Vec<String>,
}

#[derive(clap::Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
  Edit,
  Configure {
    #[arg(value_enum)]
    transport: TransportKind,
  },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TransportKind {
  Http,
}
