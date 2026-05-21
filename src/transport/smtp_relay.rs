pub struct SmtpRelay {}

impl SmtpRelay {
  pub fn new() -> Self {
    Self {}
  }
}

impl super::TransportService for SmtpRelay {
  fn send(&self, title: String, body: String) -> anyhow::Result<()> {
    Ok(())
  }
}
