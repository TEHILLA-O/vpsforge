use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistroFamily {
    Debian,
    Rhel,
    Arch,
    Suse,
    Alpine,
    Unknown,
}

impl DistroFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debian => "Debian",
            Self::Rhel => "RHEL",
            Self::Arch => "Arch",
            Self::Suse => "SUSE",
            Self::Alpine => "Alpine",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManagerKind {
    Apt,
    Dnf,
    Pacman,
    Apk,
    Zypper,
    Unknown,
}

impl PackageManagerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Apt => "apt",
            Self::Dnf => "dnf",
            Self::Pacman => "pacman",
            Self::Apk => "apk",
            Self::Zypper => "zypper",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_supported(self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InitSystem {
    Systemd,
    OpenRc,
    SysV,
    Launchd,
    Windows,
    Unknown,
}

impl InitSystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Systemd => "systemd",
            Self::OpenRc => "openrc",
            Self::SysV => "sysv",
            Self::Launchd => "launchd",
            Self::Windows => "windows",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VirtType {
    BareMetal,
    Kvm,
    Qemu,
    Xen,
    Vmware,
    HyperV,
    VirtualBox,
    OpenVz,
    Lxc,
    Docker,
    Wsl,
    Unknown,
}

impl VirtType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BareMetal => "bare-metal",
            Self::Kvm => "KVM",
            Self::Qemu => "QEMU",
            Self::Xen => "Xen",
            Self::Vmware => "VMware",
            Self::HyperV => "Hyper-V",
            Self::VirtualBox => "VirtualBox",
            Self::OpenVz => "OpenVZ",
            Self::Lxc => "LXC",
            Self::Docker => "Docker",
            Self::Wsl => "WSL",
            Self::Unknown => "unknown",
        }
    }

    pub fn environment(self) -> &'static str {
        match self {
            Self::BareMetal => "physical",
            Self::Docker | Self::Lxc | Self::OpenVz => "container",
            Self::Wsl => "WSL",
            _ => "VPS",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub distribution: String,
    pub version: String,
    pub pretty_name: String,
    pub family: DistroFamily,
    pub id_like: Vec<String>,
}

impl OsInfo {
    pub fn unknown() -> Self {
        Self {
            distribution: "Unknown".into(),
            version: String::new(),
            pretty_name: std::env::consts::OS.into(),
            family: DistroFamily::Unknown,
            id_like: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub model: String,
    pub cuda_capable: bool,
    pub driver: Option<String>,
    pub cuda_version: Option<String>,
}

impl GpuInfo {
    pub fn is_nvidia(&self) -> bool {
        self.vendor.eq_ignore_ascii_case("nvidia")
            || self.model.to_ascii_uppercase().contains("NVIDIA")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtInfo {
    pub kind: VirtType,
    pub product: Option<String>,
    pub environment: String,
}

impl VirtInfo {
    pub fn unknown() -> Self {
        Self {
            kind: VirtType::Unknown,
            product: None,
            environment: "unknown".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub public_ipv4: bool,
    pub ipv6: bool,
    pub hostname: String,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SoftwareInventory {
    pub docker: SoftwareStatus,
    pub python: SoftwareStatus,
    pub node: SoftwareStatus,
    pub postgresql: SoftwareStatus,
    pub nginx: SoftwareStatus,
    pub redis: SoftwareStatus,
    pub ollama: SoftwareStatus,
    pub uv: SoftwareStatus,
    pub git: SoftwareStatus,
    pub rustc: SoftwareStatus,
    pub go: SoftwareStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

impl Default for SoftwareStatus {
    fn default() -> Self {
        Self {
            installed: false,
            version: None,
            path: None,
        }
    }
}

impl SoftwareStatus {
    pub fn missing() -> Self {
        Self::default()
    }

    pub fn present(version: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            installed: true,
            version: Some(version.into()),
            path: Some(path.into()),
        }
    }

    pub fn display(&self) -> String {
        if !self.installed {
            return "Not installed".into();
        }
        self.version
            .clone()
            .unwrap_or_else(|| "Installed".into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostFacts {
    pub hostname: String,
    pub os: OsInfo,
    pub kernel: String,
    pub arch: String,
    pub cpu_cores: u32,
    pub cpu_brand: String,
    pub ram_bytes: u64,
    pub ram_available: u64,
    pub disk_bytes: u64,
    pub disk_available: u64,
    pub disk_kind: String,
    pub gpu: Option<GpuInfo>,
    pub virt: VirtInfo,
    pub pkg: PackageManagerKind,
    pub init: InitSystem,
    pub network: NetworkInfo,
    pub software: SoftwareInventory,
    pub linux: bool,
    pub privileged: bool,
}

impl HostFacts {
    pub fn ram_gib(&self) -> f64 {
        self.ram_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    }

    pub fn ram_available_gib(&self) -> f64 {
        self.ram_available as f64 / 1024.0 / 1024.0 / 1024.0
    }

    pub fn disk_gib(&self) -> f64 {
        self.disk_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    }

    pub fn disk_available_gib(&self) -> f64 {
        self.disk_available as f64 / 1024.0 / 1024.0 / 1024.0
    }

    pub fn has_nvidia(&self) -> bool {
        self.gpu.as_ref().is_some_and(GpuInfo::is_nvidia)
    }

    pub fn cuda_capable(&self) -> bool {
        self.gpu.as_ref().is_some_and(|g| g.cuda_capable)
    }

    pub fn ram_mib(&self) -> u64 {
        self.ram_bytes / 1024 / 1024
    }

    pub fn supports_installs(&self) -> bool {
        self.linux && self.pkg.is_supported()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nvidia_detection() {
        let gpu = GpuInfo {
            vendor: "NVIDIA".into(),
            model: "NVIDIA L4".into(),
            cuda_capable: true,
            driver: None,
            cuda_version: None,
        };
        assert!(gpu.is_nvidia());
        assert!(gpu.cuda_capable);
    }
}
