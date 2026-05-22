use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use rsdns::clients::{ClientConfig, std::Client};
use rsdns::records::RecordSet;
use rsdns::records::{Class, data::Mx};

pub fn get_mx_records(email: &str) -> anyhow::Result<Vec<Mx>> {
  let domain = email
    .split('@')
    .nth(1)
    .ok_or_else(|| anyhow::anyhow!("Invalid email address: {}", email))?;
  let mut mx_records = get_all_mx_records(domain)?;
  mx_records
    .rdata
    .sort_unstable_by(|a, b| a.preference.cmp(&b.preference));
  Ok(mx_records.rdata)
}

fn get_all_mx_records(domain: &str) -> anyhow::Result<RecordSet<Mx>> {
  let nameserver = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
  let mut client = Client::new(ClientConfig::with_nameserver(nameserver))?;
  let response = client.query_rrset::<Mx>(domain, Class::IN)?;
  Ok(response)
}
