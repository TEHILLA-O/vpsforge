use std::collections::HashMap;
use std::sync::OnceLock;

use vpsforge_core::PackageManagerKind;

#[derive(Debug, Clone)]
pub struct LogicalPackage {
    pub id: String,
    pub description: String,
    pub binaries: Vec<String>,
    pub apt: Vec<String>,
    pub dnf: Vec<String>,
    pub pacman: Vec<String>,
    pub apk: Vec<String>,
    pub zypper: Vec<String>,
    pub estimated_bytes: u64,
}

impl LogicalPackage {
    pub fn native(&self, kind: PackageManagerKind) -> Vec<String> {
        match kind {
            PackageManagerKind::Apt => self.apt.clone(),
            PackageManagerKind::Dnf => self.dnf.clone(),
            PackageManagerKind::Pacman => self.pacman.clone(),
            PackageManagerKind::Apk => self.apk.clone(),
            PackageManagerKind::Zypper => self.zypper.clone(),
            PackageManagerKind::Unknown => self.apt.clone(),
        }
    }
}

fn entry(
    id: &str,
    description: &str,
    binaries: &[&str],
    apt: &[&str],
    dnf: &[&str],
    pacman: &[&str],
    estimated: u64,
) -> LogicalPackage {
    LogicalPackage {
        id: id.into(),
        description: description.into(),
        binaries: binaries.iter().map(|s| s.to_string()).collect(),
        apt: apt.iter().map(|s| s.to_string()).collect(),
        dnf: dnf.iter().map(|s| s.to_string()).collect(),
        pacman: pacman.iter().map(|s| s.to_string()).collect(),
        apk: apt.iter().map(|s| s.to_string()).collect(),
        zypper: dnf.iter().map(|s| s.to_string()).collect(),
        estimated_bytes: estimated,
    }
}

pub fn package_catalog() -> &'static HashMap<String, LogicalPackage> {
    static CATALOG: OnceLock<HashMap<String, LogicalPackage>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let items = vec![
            entry("git", "Git", &["git"], &["git"], &["git"], &["git"], 30 * MB),
            entry("curl", "curl", &["curl"], &["curl"], &["curl"], &["curl"], 2 * MB),
            entry("wget", "wget", &["wget"], &["wget"], &["wget"], &["wget"], 2 * MB),
            entry("jq", "jq", &["jq"], &["jq"], &["jq"], &["jq"], 1 * MB),
            entry("tmux", "tmux", &["tmux"], &["tmux"], &["tmux"], &["tmux"], 2 * MB),
            entry("htop", "htop", &["htop"], &["htop"], &["htop"], &["htop"], 1 * MB),
            entry("rsync", "rsync", &["rsync"], &["rsync"], &["rsync"], &["rsync"], 2 * MB),
            entry("unzip", "unzip", &["unzip"], &["unzip"], &["unzip"], &["unzip"], 1 * MB),
            entry(
                "ca-certificates",
                "CA certificates",
                &[],
                &["ca-certificates"],
                &["ca-certificates"],
                &["ca-certificates"],
                1 * MB,
            ),
            entry(
                "build-essential",
                "C/C++ build tools",
                &["gcc", "make"],
                &["build-essential"],
                &["gcc", "gcc-c++", "make"],
                &["base-devel"],
                180 * MB,
            ),
            entry(
                "python3",
                "Python 3",
                &["python3", "python"],
                &["python3", "python3-venv", "python3-pip"],
                &["python3", "python3-pip"],
                &["python", "python-pip"],
                40 * MB,
            ),
            entry("uv", "uv Python package manager", &["uv"], &[], &[], &[], 20 * MB),
            entry(
                "nodejs",
                "Node.js",
                &["node"],
                &["nodejs", "npm"],
                &["nodejs", "npm"],
                &["nodejs", "npm"],
                60 * MB,
            ),
            entry(
                "openjdk",
                "Java",
                &["java"],
                &["openjdk-21-jdk"],
                &["java-21-openjdk-devel"],
                &["jdk-openjdk"],
                300 * MB,
            ),
            entry("golang", "Go", &["go"], &["golang"], &["golang"], &["go"], 120 * MB),
            entry(
                "docker",
                "Docker Engine",
                &["docker"],
                &["docker.io", "docker-compose-v2"],
                &["docker", "docker-compose"],
                &["docker", "docker-compose"],
                250 * MB,
            ),
            entry(
                "docker-compose",
                "Docker Compose",
                &["docker-compose", "docker"],
                &["docker-compose-v2"],
                &["docker-compose"],
                &["docker-compose"],
                20 * MB,
            ),
            entry("podman", "Podman", &["podman"], &["podman"], &["podman"], &["podman"], 80 * MB),
            entry("nginx", "Nginx", &["nginx"], &["nginx"], &["nginx"], &["nginx"], 8 * MB),
            entry("caddy", "Caddy", &["caddy"], &["caddy"], &["caddy"], &["caddy"], 20 * MB),
            entry(
                "certbot",
                "Certbot",
                &["certbot"],
                &["certbot", "python3-certbot-nginx"],
                &["certbot", "python3-certbot-nginx"],
                &["certbot"],
                20 * MB,
            ),
            entry(
                "postgresql",
                "PostgreSQL",
                &["psql", "postgres"],
                &["postgresql", "postgresql-contrib"],
                &["postgresql-server", "postgresql"],
                &["postgresql"],
                80 * MB,
            ),
            entry("redis", "Redis", &["redis-server", "redis-cli"], &["redis"], &["redis"], &["redis"], 8 * MB),
            entry(
                "mariadb",
                "MariaDB",
                &["mariadb", "mysql"],
                &["mariadb-server"],
                &["mariadb-server"],
                &["mariadb"],
                180 * MB,
            ),
            entry("mongodb", "MongoDB tools", &["mongod"], &["mongodb"], &["mongodb"], &["mongodb"], 200 * MB),
            entry("sqlite", "SQLite tools", &["sqlite3"], &["sqlite3"], &["sqlite"], &["sqlite"], 3 * MB),
            entry("minio", "MinIO", &["minio"], &[], &[], &[], 50 * MB),
            entry(
                "fail2ban",
                "Fail2ban",
                &["fail2ban-client", "fail2ban-server"],
                &["fail2ban"],
                &["fail2ban"],
                &["fail2ban"],
                8 * MB,
            ),
            entry("auditd", "auditd", &["auditctl"], &["auditd"], &["audit"], &["audit"], 4 * MB),
            entry("lynis", "Lynis", &["lynis"], &["lynis"], &["lynis"], &["lynis"], 4 * MB),
            entry(
                "clamav",
                "ClamAV",
                &["clamscan"],
                &["clamav", "clamav-daemon"],
                &["clamav", "clamav-update"],
                &["clamav"],
                400 * MB,
            ),
            entry("tcpdump", "tcpdump", &["tcpdump"], &["tcpdump"], &["tcpdump"], &["tcpdump"], 2 * MB),
            entry("nmap", "nmap", &["nmap"], &["nmap"], &["nmap"], &["nmap"], 20 * MB),
            entry(
                "tshark",
                "tshark",
                &["tshark"],
                &["tshark"],
                &["wireshark-cli"],
                &["wireshark-cli"],
                40 * MB,
            ),
            entry("ufw", "UFW firewall", &["ufw"], &["ufw"], &[], &[], 1 * MB),
            entry(
                "unattended-upgrades",
                "Automatic security updates",
                &[],
                &["unattended-upgrades"],
                &["dnf-automatic"],
                &[],
                2 * MB,
            ),
            entry("logrotate", "logrotate", &["logrotate"], &["logrotate"], &["logrotate"], &["logrotate"], 1 * MB),
            entry(
                "nvidia-container-toolkit",
                "NVIDIA Container Toolkit",
                &["nvidia-container-toolkit", "nvidia-ctk"],
                &[],
                &[],
                &[],
                40 * MB,
            ),
            entry("ollama", "Ollama", &["ollama"], &[], &[], &[], 80 * MB),
            entry("jupyter", "Jupyter", &["jupyter"], &[], &[], &[], 80 * MB),
            entry(
                "huggingface-cli",
                "Hugging Face CLI",
                &["huggingface-cli", "hf"],
                &[],
                &[],
                &[],
                20 * MB,
            ),
            entry(
                "nvtop",
                "GPU monitor",
                &["nvtop"],
                &["nvtop"],
                &["nvtop"],
                &["nvtop"],
                2 * MB,
            ),
            entry(
                "crowdsec",
                "CrowdSec",
                &["cscli"],
                &["crowdsec"],
                &["crowdsec"],
                &["crowdsec"],
                40 * MB,
            ),
            entry(
                "restic",
                "Restic backups",
                &["restic"],
                &["restic"],
                &["restic"],
                &["restic"],
                10 * MB,
            ),
        ];
        items.into_iter().map(|p| (p.id.clone(), p)).collect()
    })
}

const MB: u64 = 1024 * 1024;

pub fn native_packages(logical: &str, kind: PackageManagerKind) -> Vec<String> {
    package_catalog()
        .get(logical)
        .map(|p| p.native(kind))
        .unwrap_or_else(|| vec![logical.to_string()])
}

pub fn binaries_for(logical: &str) -> Vec<String> {
    package_catalog()
        .get(logical)
        .map(|p| p.binaries.clone())
        .unwrap_or_else(|| vec![logical.to_string()])
}

pub fn estimated_bytes(logical: &str) -> u64 {
    package_catalog()
        .get(logical)
        .map(|p| p.estimated_bytes)
        .unwrap_or(8 * MB)
}

pub fn search_catalog(query: &str) -> Vec<&'static LogicalPackage> {
    let q = query.to_ascii_lowercase();
    package_catalog()
        .values()
        .filter(|p| {
            p.id.contains(&q)
                || p.description.to_ascii_lowercase().contains(&q)
                || p.apt.iter().any(|n| n.contains(&q))
        })
        .collect()
}
