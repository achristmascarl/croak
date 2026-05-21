use clap::Parser;
use std::process::{Command, Stdio};

use crate::cli::Cli;

mod cli;
mod config;
mod process;
mod receiver;
mod utils;

fn main() -> anyhow::Result<()> {
  let args = Cli::parse();
  if let Some(command) = args.command {
    panic!("test")
  }
  println!("{:?}", args);
  if !args.target_args.is_empty() {
    let mut cmd = Command::new(&args.target_args[0]);
    for arg in &args.target_args[1..] {
      cmd.arg(arg);
    }
    let status = cmd.stdout(Stdio::inherit()) // Redirects subprocess stdout to parent stdout
      .stderr(Stdio::inherit()) // Redirects subprocess stderr to parent stderr
      .status()?;

    println!("Process finished with: {}", status);
  }
  Ok(())
}
