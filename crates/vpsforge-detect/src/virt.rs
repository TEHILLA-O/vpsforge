use std::process::Command;

use vpsforge_core::{VirtInfo, VirtType};

pub fn detect_virt() -> VirtInfo {
    if let Some(kind) = from_systemd() {
        return VirtInfo {
            environment: kind.environment().into(),
            product: Some(kind.as_str().into()),
            kind,
        };
    }
    if wsl() {
        return VirtInfo {
            kind: VirtType::Wsl,
            product: Some("WSL".into()),
            environment: "WSL".into(),
        };
    }
    if let Some(kind) = from_dmi() {
        return VirtInfo {
            environment: kind.environment().into(),
            product: Some(kind.as_str().into()),
            kind,
        };
    }
    if cfg!(windows) {
        return VirtInfo {
            kind: VirtType::Unknown,
            product: None,
            environment: "workstation".into(),
        };
    }
    VirtInfo::unknown()
}

fn from_systemd() -> Option<VirtType> {
    let output = Command::new("systemd-detect-virt").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_ascii_lowercase();
    Some(match text.as_str() {
        "none" => VirtType::BareMetal,
        "kvm" => VirtType::Kvm,
        "qemu" => VirtType::Qemu,
        "xen" => VirtType::Xen,
        "vmware" => VirtType::Vmware,
        "microsoft" | "hyperv" => VirtType::HyperV,
        "oracle" => VirtType::VirtualBox,
        "openvz" => VirtType::OpenVz,
        "lxc" | "lxc-libvirt" => VirtType::Lxc,
        "docker" | "podman" | "container" => VirtType::Docker,
        "wsl" => VirtType::Wsl,
        "" => return None,
        _ => VirtType::Unknown,
    })
}

fn wsl() -> bool {
    std::env::var("WSL_DISTRO_NAME").is_ok()
        || std::fs::read_to_string("/proc/version")
            .map(|v| v.to_ascii_lowercase().contains("microsoft"))
            .unwrap_or(false)
}

fn from_dmi() -> Option<VirtType> {
    let product = std::fs::read_to_string("/sys/class/dmi/id/product_name")
        .or_else(|_| std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let vendor = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let blob = format!("{product} {vendor}");
    if blob.contains("kvm") || blob.contains("openstack") || blob.contains("google") {
        return Some(VirtType::Kvm);
    }
    if blob.contains("qemu") {
        return Some(VirtType::Qemu);
    }
    if blob.contains("vmware") {
        return Some(VirtType::Vmware);
    }
    if blob.contains("virtualbox") || blob.contains("vbox") {
        return Some(VirtType::VirtualBox);
    }
    if blob.contains("hyper-v") || blob.contains("microsoft corporation") {
        return Some(VirtType::HyperV);
    }
    if blob.contains("xen") {
        return Some(VirtType::Xen);
    }
    None
}
