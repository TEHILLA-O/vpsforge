use regex::Regex;

#[derive(Debug, Clone)]
pub struct SshStatus {
    pub root_password_login: bool,
    pub password_auth: bool,
    pub pubkey_auth: bool,
}

pub fn ssh_status() -> SshStatus {
    let text = std::fs::read_to_string("/etc/ssh/sshd_config").unwrap_or_default();
    if text.is_empty() {
        return SshStatus {
            root_password_login: false,
            password_auth: false,
            pubkey_auth: true,
        };
    }
    SshStatus {
        root_password_login: matches_bool(&text, "PermitRootLogin", &["yes", "prohibit-password"])
            && matches_bool(&text, "PasswordAuthentication", &["yes"]),
        password_auth: matches_bool(&text, "PasswordAuthentication", &["yes"]),
        pubkey_auth: !matches_bool(&text, "PubkeyAuthentication", &["no"]),
    }
}

fn matches_bool(text: &str, key: &str, values: &[&str]) -> bool {
    let Ok(re) = Regex::new(&format!(r"(?im)^\s*{key}\s+(\S+)")) else {
        return false;
    };
    if let Some(cap) = re.captures(text) {
        let value = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        return values.iter().any(|v| v.eq_ignore_ascii_case(value));
    }
    false
}
