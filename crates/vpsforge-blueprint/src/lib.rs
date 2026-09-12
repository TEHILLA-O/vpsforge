//! Reproducible server blueprints.

mod model;

use std::path::Path;

use vpsforge_core::{ForgeError, ForgePaths, ForgeResult, HostFacts};
use vpsforge_profiles::{get, normalize_profile_id};

pub use model::Blueprint;

pub fn create(name: &str, profiles: &[String], host: &HostFacts) -> ForgeResult<Blueprint> {
    let mut bp = Blueprint::template(name);
    bp.profiles = profiles.iter().map(|p| normalize_profile_id(p)).collect();
    if bp.profiles.is_empty() {
        bp.profiles = vec!["production".into()];
    }
    for id in &bp.profiles {
        get(id)?;
    }
    bp.requirements.memory = if host.ram_gib() >= 8.0 {
        ">=8GB".into()
    } else {
        format!(">={:.0}GB", host.ram_gib().max(1.0))
    };
    bp.ai.gpu = if host.has_nvidia() {
        "auto".into()
    } else {
        "none".into()
    };
    Ok(bp)
}

pub fn validate(blueprint: &Blueprint) -> ForgeResult<()> {
    if blueprint.name.trim().is_empty() {
        return Err(ForgeError::Blueprint("blueprint name is empty".into()));
    }
    if blueprint.profiles.is_empty() {
        return Err(ForgeError::Blueprint(
            "blueprint must include at least one profile".into(),
        ));
    }
    for id in &blueprint.profiles {
        get(id)?;
    }
    if !matches!(
        blueprint.base.family.to_ascii_lowercase().as_str(),
        "debian" | "rhel" | "arch" | "suse" | "alpine"
    ) {
        return Err(ForgeError::Blueprint(format!(
            "unsupported base family '{}'",
            blueprint.base.family
        )));
    }
    Ok(())
}

pub fn save(paths: &ForgePaths, blueprint: &Blueprint) -> ForgeResult<std::path::PathBuf> {
    paths.ensure()?;
    validate(blueprint)?;
    let path = paths.blueprint_file(&blueprint.name);
    let yaml = serde_yaml::to_string(blueprint)
        .map_err(|e| ForgeError::Blueprint(e.to_string()))?;
    std::fs::write(&path, yaml)?;
    Ok(path)
}

pub fn load(paths: &ForgePaths, name: &str) -> ForgeResult<Blueprint> {
    load_file(&paths.blueprint_file(name))
}

pub fn load_file(path: &Path) -> ForgeResult<Blueprint> {
    let text = std::fs::read_to_string(path)?;
    let bp: Blueprint =
        serde_yaml::from_str(&text).map_err(|e| ForgeError::Blueprint(e.to_string()))?;
    validate(&bp)?;
    Ok(bp)
}

pub fn list(paths: &ForgePaths) -> ForgeResult<Vec<String>> {
    let mut names = Vec::new();
    if !paths.blueprint_dir.exists() {
        return Ok(names);
    }
    for entry in std::fs::read_dir(&paths.blueprint_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn render(blueprint: &Blueprint) -> String {
    serde_yaml::to_string(blueprint).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vpsforge_core::DistroFamily;

    fn host() -> HostFacts {
        let h = vpsforge_core::HostFacts {
            hostname: "test".into(),
            os: vpsforge_core::OsInfo {
                distribution: "Ubuntu".into(),
                version: "24.04".into(),
                pretty_name: "Ubuntu 24.04".into(),
                family: DistroFamily::Debian,
                id_like: vec!["debian".into()],
            },
            kernel: "6.8".into(),
            arch: "x86_64".into(),
            cpu_cores: 8,
            cpu_brand: "test".into(),
            ram_bytes: 16 * 1024 * 1024 * 1024,
            ram_available: 14 * 1024 * 1024 * 1024,
            disk_bytes: 100 * 1024 * 1024 * 1024,
            disk_available: 80 * 1024 * 1024 * 1024,
            disk_kind: "NVMe".into(),
            gpu: None,
            virt: vpsforge_core::VirtInfo::unknown(),
            pkg: vpsforge_core::PackageManagerKind::Apt,
            init: vpsforge_core::InitSystem::Systemd,
            network: vpsforge_core::NetworkInfo {
                public_ipv4: true,
                ipv6: false,
                hostname: "test".into(),
                interfaces: vec![],
            },
            software: Default::default(),
            linux: true,
            privileged: true,
        };
        h
    }

    #[test]
    fn template_validates() {
        let bp = create("ai-production", &["production".into(), "ai".into(), "devops".into()], &host())
            .unwrap();
        validate(&bp).unwrap();
        assert_eq!(bp.profiles[0], "production");
    }
}
