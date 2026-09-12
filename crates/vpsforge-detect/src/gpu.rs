use std::process::Command;

use vpsforge_core::GpuInfo;

pub fn detect_gpu() -> Option<GpuInfo> {
    if let Some(gpu) = from_nvidia_smi() {
        return Some(gpu);
    }
    if let Some(gpu) = from_lspci() {
        return Some(gpu);
    }
    from_sysfs()
}

fn from_nvidia_smi() -> Option<GpuInfo> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,driver_version",
            "--format=csv,noheader",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }
    let mut parts = line.split(',').map(|s| s.trim());
    let model = parts.next()?.to_string();
    let driver = parts.next().map(|s| s.to_string());
    let cuda = cuda_version();
    Some(GpuInfo {
        vendor: "NVIDIA".into(),
        model,
        cuda_capable: true,
        driver,
        cuda_version: cuda,
    })
}

fn cuda_version() -> Option<String> {
    let output = Command::new("nvidia-smi").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(rest) = line.split("CUDA Version:").nth(1) {
            return Some(rest.split_whitespace().next()?.to_string());
        }
    }
    None
}

fn from_lspci() -> Option<GpuInfo> {
    let output = Command::new("lspci").args(["-nn"]).output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if !(lower.contains("vga") || lower.contains("3d controller") || lower.contains("display"))
        {
            continue;
        }
        if lower.contains("nvidia") {
            return Some(GpuInfo {
                vendor: "NVIDIA".into(),
                model: extract_model(line, "NVIDIA"),
                cuda_capable: true,
                driver: None,
                cuda_version: None,
            });
        }
        if lower.contains("amd") || lower.contains("ati") {
            return Some(GpuInfo {
                vendor: "AMD".into(),
                model: extract_model(line, "AMD"),
                cuda_capable: false,
                driver: None,
                cuda_version: None,
            });
        }
        if lower.contains("intel") {
            return Some(GpuInfo {
                vendor: "Intel".into(),
                model: extract_model(line, "Intel"),
                cuda_capable: false,
                driver: None,
                cuda_version: None,
            });
        }
    }
    None
}

fn extract_model(line: &str, vendor: &str) -> String {
    if let Some(idx) = line.find(vendor) {
        let rest = line[idx..]
            .split('[')
            .next()
            .unwrap_or(line)
            .trim()
            .to_string();
        if !rest.is_empty() {
            return rest;
        }
    }
    line.trim().to_string()
}

fn from_sysfs() -> Option<GpuInfo> {
    let path = "/sys/class/drm";
    let entries = std::fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        let vendor = std::fs::read_to_string(entry.path().join("device/vendor")).ok()?;
        let vendor = vendor.trim();
        // NVIDIA PCI vendor ID
        if vendor.eq_ignore_ascii_case("0x10de") {
            return Some(GpuInfo {
                vendor: "NVIDIA".into(),
                model: "NVIDIA GPU".into(),
                cuda_capable: true,
                driver: None,
                cuda_version: None,
            });
        }
    }
    None
}
