#[derive(Debug, Clone)]
pub struct FirewallStatus {
    pub enabled: bool,
    pub backend: String,
}

pub fn firewall_status() -> FirewallStatus {
    if which::which("ufw").is_ok() {
        let output = std::process::Command::new("ufw")
            .arg("status")
            .output()
            .ok();
        if let Some(out) = output {
            let text = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
            return FirewallStatus {
                enabled: text.contains("status: active"),
                backend: "ufw".into(),
            };
        }
    }
    if which::which("firewall-cmd").is_ok() {
        let ok = std::process::Command::new("firewall-cmd")
            .arg("--state")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        return FirewallStatus {
            enabled: ok,
            backend: "firewalld".into(),
        };
    }
    if std::path::Path::new("/etc/nftables.conf").exists() {
        return FirewallStatus {
            enabled: true,
            backend: "nftables".into(),
        };
    }
    FirewallStatus {
        enabled: false,
        backend: "none".into(),
    }
}
