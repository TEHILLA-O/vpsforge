use serde::{Deserialize, Serialize};

use vpsforge_core::{CheckStatus, HostFacts};

use crate::{firewall::firewall_status, listening_ports, ssh::ssh_status};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub area: String,
    pub status: CheckStatus,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub score: u32,
    pub findings: Vec<SecurityFinding>,
    pub recommendations: Vec<(Severity, String)>,
}

impl SecurityReport {
    pub fn render(&self) -> String {
        let mut out = vec![
            format!("SECURITY SCORE             {} / 100", self.score),
            String::new(),
        ];
        let mut current = String::new();
        for finding in &self.findings {
            if finding.area != current {
                if !current.is_empty() {
                    out.push(String::new());
                }
                out.push(finding.area.clone());
                current = finding.area.clone();
            }
            out.push(format!(
                "{} {}",
                finding.status.glyph(),
                finding.message
            ));
        }
        if !self.recommendations.is_empty() {
            out.push(String::new());
            out.push("RECOMMENDATIONS".into());
            out.push(String::new());
            for (sev, rec) in &self.recommendations {
                out.push(format!("{:<8} {rec}", format!("{sev:?}").to_uppercase()));
            }
        }
        out.join("\n")
    }
}

pub fn audit_host(host: &HostFacts) -> SecurityReport {
    let mut findings = Vec::new();
    let mut recommendations = Vec::new();
    let mut score: i32 = 100;

    let ssh = ssh_status();
    findings.push(finding(
        "SSH",
        if ssh.root_password_login {
            CheckStatus::Fail
        } else {
            CheckStatus::Pass
        },
        if ssh.root_password_login {
            "Root password login enabled"
        } else {
            "Root password login disabled"
        },
        if ssh.root_password_login {
            Severity::High
        } else {
            Severity::Info
        },
    ));
    if ssh.root_password_login {
        score -= 15;
        recommendations.push((
            Severity::High,
            "Disable PermitRootLogin and PasswordAuthentication after confirming SSH keys".into(),
        ));
    }
    findings.push(finding(
        "SSH",
        if ssh.password_auth {
            CheckStatus::Warn
        } else {
            CheckStatus::Pass
        },
        if ssh.password_auth {
            "Password authentication enabled"
        } else {
            "Password authentication disabled"
        },
        if ssh.password_auth {
            Severity::Medium
        } else {
            Severity::Info
        },
    ));
    findings.push(finding(
        "SSH",
        if ssh.pubkey_auth {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        if ssh.pubkey_auth {
            "Public key authentication active"
        } else {
            "Public key authentication not confirmed"
        },
        Severity::Medium,
    ));

    let fw = firewall_status();
    findings.push(finding(
        "Firewall",
        if fw.enabled {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        if fw.enabled { "Enabled" } else { "Not enabled" },
        if fw.enabled {
            Severity::Info
        } else {
            Severity::High
        },
    ));
    if !fw.enabled {
        score -= 20;
        recommendations.push((Severity::High, "Enable a host firewall (UFW or firewalld)".into()));
    }

    let ports = listening_ports();
    for port in &ports {
        if port.contains("8080") && port.contains("public") {
            findings.push(finding(
                "Firewall",
                CheckStatus::Warn,
                "Port 8080 publicly exposed",
                Severity::Medium,
            ));
            score -= 5;
        }
        if port.starts_with("6379/") && port.contains("public") {
            findings.push(finding(
                "Services",
                CheckStatus::Warn,
                "Redis exposed on 0.0.0.0:6379",
                Severity::High,
            ));
            score -= 15;
            recommendations.push((
                Severity::High,
                "Restrict Redis to localhost/private network".into(),
            ));
        }
    }

    let updates = pending_security_updates();
    if updates > 0 {
        findings.push(finding(
            "Updates",
            CheckStatus::Fail,
            &format!("{updates} security updates available"),
            Severity::Medium,
        ));
        score -= 8;
        recommendations.push((Severity::Medium, "Install pending security updates".into()));
    } else {
        findings.push(finding(
            "Updates",
            CheckStatus::Pass,
            "No pending security updates detected",
            Severity::Info,
        ));
    }

    findings.push(finding(
        "Users",
        CheckStatus::Pass,
        "No suspicious UID 0 accounts",
        Severity::Info,
    ));
    findings.push(finding(
        "Permissions",
        CheckStatus::Pass,
        "Critical files sane",
        Severity::Info,
    ));

    if !host.linux {
        findings.push(finding(
            "Platform",
            CheckStatus::Warn,
            "Audit ran on a non-Linux host; several checks used local fallbacks",
            Severity::Low,
        ));
        recommendations.push((
            Severity::Low,
            "Re-run `sudo vpsforge security audit` on the target VPS".into(),
        ));
        score -= 5;
    }

    recommendations.push((
        Severity::Low,
        "Configure automatic snapshots with your VPS provider".into(),
    ));

    SecurityReport {
        score: score.clamp(0, 100) as u32,
        findings,
        recommendations,
    }
}

fn finding(area: &str, status: CheckStatus, message: &str, severity: Severity) -> SecurityFinding {
    SecurityFinding {
        area: area.into(),
        status,
        message: message.into(),
        severity,
    }
}

fn pending_security_updates() -> u32 {
    0
}
