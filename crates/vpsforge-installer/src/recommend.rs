use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresTuning {
    pub shared_buffers: String,
    pub effective_cache_size: String,
    pub work_mem: String,
    pub maintenance_work_mem: String,
    pub max_connections: u32,
    pub notes: String,
}

/// Conservative PostgreSQL memory guidance.
///
/// These are recommendations only. VPSForge will not overwrite
/// postgresql.conf unless the operator opts in.
pub fn postgres_tuning(ram_bytes: u64) -> PostgresTuning {
    let ram_mb = (ram_bytes / 1024 / 1024).max(256);
    let shared = (ram_mb / 4).max(128);
    let cache = (ram_mb * 5 / 8).max(256);
    let work = if ram_mb >= 16 * 1024 {
        32
    } else if ram_mb >= 8 * 1024 {
        16
    } else if ram_mb >= 4 * 1024 {
        8
    } else {
        4
    };
    let maintenance = (ram_mb / 16).clamp(64, 2048);
    PostgresTuning {
        shared_buffers: format_mb(shared),
        effective_cache_size: format_mb(cache),
        work_mem: format_mb(work),
        maintenance_work_mem: format_mb(maintenance),
        max_connections: 100,
        notes: "Do not apply blindly. Review against existing postgresql.conf and workload."
            .into(),
    }
}

fn format_mb(mb: u64) -> String {
    if mb >= 1024 && mb % 1024 == 0 {
        format!("{} GB", mb / 1024)
    } else {
        format!("{mb} MB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixteen_gb_matches_spec() {
        let tune = postgres_tuning(16 * 1024 * 1024 * 1024);
        assert_eq!(tune.shared_buffers, "4 GB");
        assert_eq!(tune.effective_cache_size, "10 GB");
        assert_eq!(tune.work_mem, "32 MB");
    }
}
