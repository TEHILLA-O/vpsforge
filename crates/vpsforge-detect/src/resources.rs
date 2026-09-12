use sysinfo::{Disks, System};

use crate::ResourceSnapshot;

pub fn detect_resources() -> ResourceSnapshot {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    let cpu_cores = sys.cpus().len().max(1) as u32;
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into());

    let disks = Disks::new_with_refreshed_list();
    let mut disk_bytes = 0u64;
    let mut disk_available = 0u64;
    let mut disk_kind = "Unknown".to_string();
    for disk in disks.list() {
        disk_bytes = disk_bytes.saturating_add(disk.total_space());
        disk_available = disk_available.saturating_add(disk.available_space());
        let name = format!("{:?}", disk.kind()).to_ascii_uppercase();
        if name.contains("NVME") || disk.name().to_string_lossy().to_ascii_lowercase().contains("nvme")
        {
            disk_kind = "NVMe".into();
        } else if name.contains("SSD") && disk_kind == "Unknown" {
            disk_kind = "SSD".into();
        } else if name.contains("HDD") && disk_kind == "Unknown" {
            disk_kind = "HDD".into();
        }
    }

    ResourceSnapshot {
        cpu_cores,
        cpu_brand,
        ram_bytes: sys.total_memory(),
        ram_available: sys.available_memory(),
        disk_bytes,
        disk_available,
        disk_kind,
        kernel: System::kernel_version().unwrap_or_else(|| "unknown".into()),
        arch: std::env::consts::ARCH.to_string(),
    }
}
