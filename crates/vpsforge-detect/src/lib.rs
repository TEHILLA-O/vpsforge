//! Host detection: OS, resources, GPU, virtualisation, network, software.

mod gpu;
mod network;
mod os;
mod resources;
mod software;
mod virt;

use vpsforge_core::{
    HostFacts, InitSystem, NetworkInfo, OsInfo, PackageManagerKind, VirtInfo, VirtType,
};

pub use gpu::detect_gpu;
pub use network::detect_network;
pub use os::{detect_init, detect_os, detect_package_manager};
pub use resources::detect_resources;
pub use software::detect_software;
pub use virt::detect_virt;

#[derive(Debug, Clone)]
pub struct ResourceSnapshot {
    pub cpu_cores: u32,
    pub cpu_brand: String,
    pub ram_bytes: u64,
    pub ram_available: u64,
    pub disk_bytes: u64,
    pub disk_available: u64,
    pub disk_kind: String,
    pub kernel: String,
    pub arch: String,
}

/// Probe the machine VPSForge is running on.
pub fn detect() -> HostFacts {
    let os = detect_os();
    let resources = detect_resources();
    let gpu = detect_gpu();
    let virt = detect_virt();
    let pkg = detect_package_manager(&os);
    let init = detect_init();
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".into());
    let network = detect_network(&hostname);
    let software = detect_software();

    HostFacts {
        hostname,
        os,
        kernel: resources.kernel,
        arch: resources.arch,
        cpu_cores: resources.cpu_cores,
        cpu_brand: resources.cpu_brand,
        ram_bytes: resources.ram_bytes,
        ram_available: resources.ram_available,
        disk_bytes: resources.disk_bytes,
        disk_available: resources.disk_available,
        disk_kind: resources.disk_kind,
        gpu,
        virt,
        pkg,
        init,
        network,
        software,
        linux: cfg!(target_os = "linux"),
        privileged: is_privileged(),
    }
}

fn is_privileged() -> bool {
    #[cfg(unix)]
    {
        nix::unistd::Uid::effective().is_root()
    }
    #[cfg(windows)]
    {
        false
    }
}

/// Build a synthetic host used by tests and plan previews.
pub fn sample_host() -> HostFacts {
    HostFacts {
        hostname: "ubuntu-vps-01".into(),
        os: OsInfo {
            distribution: "Ubuntu".into(),
            version: "24.04".into(),
            pretty_name: "Ubuntu 24.04 LTS".into(),
            family: vpsforge_core::DistroFamily::Debian,
            id_like: vec!["debian".into()],
        },
        kernel: "6.8.0-generic".into(),
        arch: "x86_64".into(),
        cpu_cores: 8,
        cpu_brand: "QEMU Virtual CPU".into(),
        ram_bytes: 16 * 1024 * 1024 * 1024,
        ram_available: (14.7 * 1024.0 * 1024.0 * 1024.0) as u64,
        disk_bytes: 160 * 1024 * 1024 * 1024,
        disk_available: 148 * 1024 * 1024 * 1024,
        disk_kind: "NVMe".into(),
        gpu: Some(vpsforge_core::GpuInfo {
            vendor: "NVIDIA".into(),
            model: "NVIDIA L4".into(),
            cuda_capable: true,
            driver: Some("550.54".into()),
            cuda_version: Some("12.4".into()),
        }),
        virt: VirtInfo {
            kind: VirtType::Kvm,
            product: Some("KVM".into()),
            environment: "VPS".into(),
        },
        pkg: PackageManagerKind::Apt,
        init: InitSystem::Systemd,
        network: NetworkInfo {
            public_ipv4: true,
            ipv6: true,
            hostname: "ubuntu-vps-01".into(),
            interfaces: vec!["eth0".into()],
        },
        software: {
            let mut s = vpsforge_core::SoftwareInventory::default();
            s.python = vpsforge_core::SoftwareStatus::present("3.12", "/usr/bin/python3");
            s.nginx = vpsforge_core::SoftwareStatus::present("1.24", "/usr/sbin/nginx");
            s.git = vpsforge_core::SoftwareStatus::present("2.43", "/usr/bin/git");
            s
        },
        linux: true,
        privileged: true,
    }
}

pub fn constrained_host() -> HostFacts {
    let mut host = sample_host();
    host.cpu_cores = 2;
    host.ram_bytes = 2 * 1024 * 1024 * 1024;
    host.ram_available = (1.4 * 1024.0 * 1024.0 * 1024.0) as u64;
    host.disk_bytes = 38 * 1024 * 1024 * 1024;
    host.disk_available = 30 * 1024 * 1024 * 1024;
    host.gpu = None;
    host.hostname = "tiny-vps".into();
    host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_does_not_panic() {
        let facts = detect();
        assert!(!facts.hostname.is_empty());
        assert!(facts.cpu_cores >= 1);
    }
}
