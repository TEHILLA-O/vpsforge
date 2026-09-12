//! Defensive security auditing. No offensive exploit helpers.

mod audit;
mod firewall;
mod ssh;

use vpsforge_core::HostFacts;

pub use audit::{audit_host, SecurityFinding, SecurityReport};
pub use firewall::firewall_status;
pub use ssh::ssh_status;

pub fn network_scan_summary(host: &HostFacts) -> String {
    let ports = listening_ports();
    let mut lines = vec![
        "VPSFORGE NETWORK SCAN".into(),
        String::new(),
        "Listening sockets (local observation only)".into(),
        "────────────────────────────────────────".into(),
    ];
    if ports.is_empty() {
        lines.push("No listening TCP ports discovered from local tables.".into());
    } else {
        for port in ports {
            lines.push(format!("  {port}"));
        }
    }
    if host.network.public_ipv4 {
        lines.push(String::new());
        lines.push("Public IPv4 route: detected".into());
    }
    if host.network.ipv6 {
        lines.push("IPv6: available".into());
    }
    lines.join("\n")
}

pub fn listening_ports() -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(text) = std::fs::read_to_string("/proc/net/tcp") {
        out.extend(parse_proc_net_tcp(&text, false));
    }
    if let Ok(text) = std::fs::read_to_string("/proc/net/tcp6") {
        out.extend(parse_proc_net_tcp(&text, true));
    }
    out.sort();
    out.dedup();
    out
}

fn parse_proc_net_tcp(text: &str, ipv6: bool) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        // state 0A = LISTEN
        if cols[3] != "0A" {
            continue;
        }
        let Some((addr, port_hex)) = cols[1].rsplit_once(':') else {
            continue;
        };
        let port = u16::from_str_radix(port_hex, 16).unwrap_or(0);
        let exposed = if ipv6 {
            addr != "00000000000000000000000000000000" && addr != "0000000000000000FFFF000000000000"
        } else {
            addr == "00000000"
        };
        let bind = if ipv6 { "tcp6" } else { "tcp" };
        let scope = if exposed { "public" } else { "local" };
        out.push(format!("{port}/{bind} ({scope})"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_listen_line() {
        let sample = "  sl  local_address rem_address   st\n   0: 00000000:0050 00000000:0000 0A\n";
        let ports = parse_proc_net_tcp(sample, false);
        assert!(ports.iter().any(|p| p.starts_with("80/tcp")));
    }
}
