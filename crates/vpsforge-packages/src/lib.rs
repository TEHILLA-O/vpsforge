//! Package-manager abstraction: one logical name, many distros.

mod mapping;
mod trust;
mod managers;

use vpsforge_core::{ForgeResult, PackageManagerKind};

pub use mapping::{
    estimated_bytes, native_packages, package_catalog, search_catalog, LogicalPackage,
};
pub use managers::{command_for, PackageCommand};
pub use trust::{known_sources, source_for, verify_source};

pub trait PackageManager {
    fn kind(&self) -> PackageManagerKind;
    fn is_installed(&self, logical: &str) -> bool;
    fn install_cmd(&self, native: &[String]) -> PackageCommand;
    fn remove_cmd(&self, native: &[String]) -> PackageCommand;
    fn search_cmd(&self, query: &str) -> PackageCommand;
    fn update_index_cmd(&self) -> PackageCommand;
    fn upgrade_cmd(&self) -> PackageCommand;
    fn pending_updates(&self) -> ForgeResult<UpdateSummary>;
}

#[derive(Debug, Clone, Default)]
pub struct UpdateSummary {
    pub total: u32,
    pub security: u32,
}

pub fn manager(kind: PackageManagerKind) -> Box<dyn PackageManager> {
    match kind {
        PackageManagerKind::Apt => Box::new(managers::Apt),
        PackageManagerKind::Dnf => Box::new(managers::Dnf),
        PackageManagerKind::Pacman => Box::new(managers::Pacman),
        PackageManagerKind::Zypper => Box::new(managers::Zypper),
        PackageManagerKind::Apk => Box::new(managers::Apk),
        PackageManagerKind::Unknown => Box::new(managers::Unsupported),
    }
}

pub fn logical_installed(logical: &str) -> bool {
    mapping::binaries_for(logical)
        .iter()
        .any(|bin| which::which(bin).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nginx_maps_everywhere() {
        let apt = native_packages("nginx", PackageManagerKind::Apt);
        assert_eq!(apt, vec!["nginx".to_string()]);
        let dnf = native_packages("build-essential", PackageManagerKind::Dnf);
        assert!(dnf.contains(&"gcc".to_string()));
    }
}
