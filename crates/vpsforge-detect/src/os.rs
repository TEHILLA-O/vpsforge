use std::collections::HashMap;
use std::path::Path;

use vpsforge_core::{DistroFamily, InitSystem, OsInfo, PackageManagerKind};

pub fn detect_os() -> OsInfo {
    if let Some(info) = parse_os_release("/etc/os-release") {
        return info;
    }
    if let Some(info) = parse_os_release("/usr/lib/os-release") {
        return info;
    }

    #[cfg(windows)]
    {
        OsInfo {
            distribution: "Windows".into(),
            version: std::env::var("OS").unwrap_or_else(|_| "Windows".into()),
            pretty_name: "Windows".into(),
            family: DistroFamily::Unknown,
            id_like: Vec::new(),
        }
    }
    #[cfg(not(windows))]
    {
        OsInfo::unknown()
    }
}

fn parse_os_release(path: impl AsRef<Path>) -> Option<OsInfo> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        let value = value.trim_matches('"').trim_matches('\'').to_string();
        map.insert(key.to_string(), value);
    }

    let id = map
        .get("ID")
        .cloned()
        .unwrap_or_else(|| "unknown".into())
        .to_ascii_lowercase();
    let id_like = map
        .get("ID_LIKE")
        .map(|s| {
            s.split_whitespace()
                .map(|v| v.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let family = family_from(&id, &id_like);
    Some(OsInfo {
        distribution: pretty_distro(&id, map.get("NAME").cloned()),
        version: map
            .get("VERSION_ID")
            .cloned()
            .or_else(|| map.get("VERSION").cloned())
            .unwrap_or_default(),
        pretty_name: map
            .get("PRETTY_NAME")
            .cloned()
            .unwrap_or_else(|| id.clone()),
        family,
        id_like,
    })
}

fn pretty_distro(id: &str, name: Option<String>) -> String {
    match id {
        "ubuntu" => "Ubuntu".into(),
        "debian" => "Debian".into(),
        "fedora" => "Fedora".into(),
        "rhel" => "RHEL".into(),
        "centos" => "CentOS".into(),
        "rocky" => "Rocky Linux".into(),
        "almalinux" => "AlmaLinux".into(),
        "arch" => "Arch Linux".into(),
        "opensuse-leap" | "opensuse-tumbleweed" | "opensuse" => "openSUSE".into(),
        "alpine" => "Alpine".into(),
        _ => name.unwrap_or_else(|| id.to_string()),
    }
}

fn family_from(id: &str, id_like: &[String]) -> DistroFamily {
    let tokens: Vec<&str> = std::iter::once(id)
        .chain(id_like.iter().map(String::as_str))
        .collect();
    if tokens.iter().any(|t| {
        matches!(
            *t,
            "debian" | "ubuntu" | "linuxmint" | "pop" | "raspbian" | "elementary"
        )
    }) {
        return DistroFamily::Debian;
    }
    if tokens.iter().any(|t| {
        matches!(
            *t,
            "rhel" | "fedora" | "centos" | "rocky" | "almalinux" | "ol" | "amzn"
        )
    }) {
        return DistroFamily::Rhel;
    }
    if tokens.iter().any(|t| matches!(*t, "arch" | "manjaro" | "endeavouros")) {
        return DistroFamily::Arch;
    }
    if tokens
        .iter()
        .any(|t| matches!(*t, "suse" | "opensuse" | "opensuse-leap" | "opensuse-tumbleweed"))
    {
        return DistroFamily::Suse;
    }
    if tokens.iter().any(|t| *t == "alpine") {
        return DistroFamily::Alpine;
    }
    DistroFamily::Unknown
}

pub fn detect_package_manager(os: &OsInfo) -> PackageManagerKind {
    if which::which("apt-get").is_ok() || which::which("apt").is_ok() {
        return PackageManagerKind::Apt;
    }
    if which::which("dnf").is_ok() {
        return PackageManagerKind::Dnf;
    }
    if which::which("pacman").is_ok() {
        return PackageManagerKind::Pacman;
    }
    if which::which("zypper").is_ok() {
        return PackageManagerKind::Zypper;
    }
    if which::which("apk").is_ok() {
        return PackageManagerKind::Apk;
    }
    match os.family {
        DistroFamily::Debian => PackageManagerKind::Apt,
        DistroFamily::Rhel => PackageManagerKind::Dnf,
        DistroFamily::Arch => PackageManagerKind::Pacman,
        DistroFamily::Suse => PackageManagerKind::Zypper,
        DistroFamily::Alpine => PackageManagerKind::Apk,
        DistroFamily::Unknown => PackageManagerKind::Unknown,
    }
}

pub fn detect_init() -> InitSystem {
    if cfg!(windows) {
        return InitSystem::Windows;
    }
    if Path::new("/run/systemd/system").exists() || which::which("systemctl").is_ok() {
        return InitSystem::Systemd;
    }
    if Path::new("/sbin/openrc").exists() || Path::new("/etc/init.d/openrc").exists() {
        return InitSystem::OpenRc;
    }
    if Path::new("/etc/init.d").exists() {
        return InitSystem::SysV;
    }
    InitSystem::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debian_family() {
        assert_eq!(
            family_from("ubuntu", &["debian".into()]),
            DistroFamily::Debian
        );
        assert_eq!(family_from("fedora", &[]), DistroFamily::Rhel);
        assert_eq!(family_from("arch", &[]), DistroFamily::Arch);
    }
}
