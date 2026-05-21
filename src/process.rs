pub fn run(target: Vec<String>) -> anyhow::Result<()> {
  if target.is_empty() {
    anyhow::bail!("No target command provided. Nothing to run.");
  }
  let mut cmd = std::process::Command::new(&target[0]);
  for arg in &target[1..] {
    cmd.arg(arg);
  }
  let status = cmd.stdout(std::process::Stdio::inherit()) // Redirects subprocess stdout to parent stdout
      .stderr(std::process::Stdio::inherit()) // Redirects subprocess stderr to parent stderr
      .status()?;
  println!("Process finished with: {}", status);
  Ok(())
}
