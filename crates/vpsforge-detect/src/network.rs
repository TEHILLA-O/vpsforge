use vpsforge_core::NetworkInfo;

pub fn detect_network(hostname: &str) -> NetworkInfo {
    let mut interfaces = Vec::new();
    let mut ipv6 = false;

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == "lo" {
                continue;
            }
            interfaces.push(name);
        }
    }

    // Cheap local signal: any inet6 addr besides loopback.
    if let Ok(text) = std::fs::read_to_string("/proc/net/if_inet6") {
        ipv6 = text.lines().any(|line| {
            let iface = line.split_whitespace().nth(5).unwrap_or("");
            iface != "lo"
        });
    }

    let public_ipv4 = has_default_route();

    NetworkInfo {
        public_ipv4,
        ipv6,
        hostname: hostname.to_string(),
        interfaces,
    }
}

fn has_default_route() -> bool {
    if let Ok(text) = std::fs::read_to_string("/proc/net/route") {
        return text.lines().skip(1).any(|line| {
            line.split_whitespace()
                .nth(1)
                .is_some_and(|dest| dest == "00000000")
        });
    }
    true
}
