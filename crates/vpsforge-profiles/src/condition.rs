use serde::{Deserialize, Serialize};

use vpsforge_core::HostFacts;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Condition {
    #[serde(default)]
    pub gpu_vendor: Option<String>,
    #[serde(default)]
    pub not_gpu_vendor: Option<String>,
    #[serde(default)]
    pub min_ram_mb: Option<u64>,
    #[serde(default)]
    pub min_cpu: Option<u32>,
    #[serde(default)]
    pub min_disk_gb: Option<u64>,
    #[serde(default)]
    pub cuda: Option<bool>,
}

impl Condition {
    pub fn matches(&self, host: &HostFacts) -> bool {
        self.skip_reason(host).is_none()
    }

    pub fn skip_reason(&self, host: &HostFacts) -> Option<String> {
        if let Some(vendor) = &self.gpu_vendor {
            match &host.gpu {
                Some(gpu) if gpu.vendor.eq_ignore_ascii_case(vendor) => {}
                Some(gpu) => {
                    return Some(format!(
                        "requires {vendor} GPU (detected {})",
                        gpu.vendor
                    ))
                }
                None => return Some(format!("requires {vendor} GPU (none detected)")),
            }
        }
        if let Some(vendor) = &self.not_gpu_vendor {
            if host
                .gpu
                .as_ref()
                .is_some_and(|g| g.vendor.eq_ignore_ascii_case(vendor))
            {
                return Some(format!("skipped because {vendor} GPU is present"));
            }
        }
        if let Some(min) = self.min_ram_mb {
            if host.ram_mib() < min {
                return Some(format!(
                    "requires {min} MB RAM (detected {} MB)",
                    host.ram_mib()
                ));
            }
        }
        if let Some(min) = self.min_cpu {
            if host.cpu_cores < min {
                return Some(format!(
                    "requires {min} CPU cores (detected {})",
                    host.cpu_cores
                ));
            }
        }
        if let Some(min) = self.min_disk_gb {
            if host.disk_gib() < min as f64 {
                return Some(format!(
                    "requires {min} GB disk (detected {:.0} GB)",
                    host.disk_gib()
                ));
            }
        }
        if self.cuda == Some(true) && !host.cuda_capable() {
            return Some("requires CUDA-capable GPU".into());
        }
        None
    }
}
