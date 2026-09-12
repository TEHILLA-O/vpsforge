use indexmap::IndexSet;

use vpsforge_core::{
    ActionKind, ForgeResult, HostFacts, InstallPlan, PlanWarning, PlannedAction, SkippedAction,
    WarningLevel,
};
use vpsforge_packages::{estimated_bytes, logical_installed, native_packages, source_for};
use vpsforge_profiles::{
    evaluate_profile, ordered_components, Component, ComponentKind, EvaluatedProfile, Profile,
    ProfileFlags,
};

use crate::recommend::postgres_tuning;

pub fn build_plan(
    host: &HostFacts,
    profiles: &[Profile],
    flags: &ProfileFlags,
) -> ForgeResult<InstallPlan> {
    let evaluated: Vec<EvaluatedProfile> = profiles
        .iter()
        .map(|p| evaluate_profile(p, host, flags))
        .collect();

    let mut skipped = Vec::new();
    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();
    let mut actions = Vec::new();
    let mut firewall_ports = IndexSet::new();

    for ev in &evaluated {
        for (component, reason) in ev.skipped_components() {
            skipped.push(SkippedAction {
                id: component.id.clone(),
                display: component.display_name(),
                reason,
            });
        }
    }

    let included_profiles: Vec<Profile> = evaluated
        .iter()
        .map(|ev| {
            let mut profile = ev.profile.clone();
            let included_ids: IndexSet<_> = ev
                .included_components()
                .into_iter()
                .map(|c| c.id.clone())
                .collect();
            for section in &mut profile.sections {
                section
                    .components
                    .retain(|c| included_ids.contains(&c.id));
            }
            profile
        })
        .collect();

    let ordered = ordered_components(&included_profiles).map_err(vpsforge_core::ForgeError::msg)?;

    for node in ordered {
        let component = &node.component;
        actions.extend(actions_for_component(
            host,
            &node.profile,
            &node.section,
            component,
            &mut firewall_ports,
        ));
        if component.tune.as_deref() == Some("postgres") {
            let tune = postgres_tuning(host.ram_bytes);
            recommendations.push(format!(
                "PostgreSQL suggested allocation for {:.0} GB RAM: shared_buffers={}, effective_cache_size={}, work_mem={}",
                host.ram_gib(),
                tune.shared_buffers,
                tune.effective_cache_size,
                tune.work_mem
            ));
            recommendations.push(
                "Recommendations are not applied automatically. Use --apply-tuning to write them."
                    .into(),
            );
        }
    }

    add_profile_warnings(host, profiles, &mut warnings);

    let repos = actions
        .iter()
        .filter(|a| a.kind == ActionKind::AddRepository && a.pending())
        .count();
    let services = actions
        .iter()
        .filter(|a| a.kind == ActionKind::EnableService && a.pending())
        .count();
    let config_files = actions
        .iter()
        .filter(|a| a.kind == ActionKind::WriteConfig && a.pending())
        .count();
    let estimated_bytes = actions
        .iter()
        .filter(|a| a.pending())
        .map(|a| a.estimated_bytes)
        .sum();

    let names: Vec<String> = profiles.iter().map(|p| p.id.clone()).collect();
    Ok(InstallPlan {
        title: plan_title(&names),
        profiles: names,
        actions,
        skipped,
        warnings,
        recommendations,
        estimated_bytes,
        repos,
        services,
        firewall_ports: firewall_ports.into_iter().collect(),
        config_files,
    })
}

fn actions_for_component(
    host: &HostFacts,
    profile: &str,
    section: &str,
    component: &Component,
    firewall_ports: &mut IndexSet<String>,
) -> Vec<PlannedAction> {
    let mut out = Vec::new();
    let installed = component
        .packages
        .iter()
        .all(|p| p.is_empty() || logical_installed(p))
        && !component.packages.is_empty();

    match component.kind {
        ComponentKind::Check => {
            out.push(PlannedAction {
                id: component.id.clone(),
                kind: ActionKind::RunCommand,
                target: component.id.clone(),
                display: component.display_name(),
                section: section.into(),
                profile: profile.into(),
                reason: component
                    .notes
                    .clone()
                    .unwrap_or_else(|| "validation check".into()),
                already_satisfied: false,
                reversibility: component.reversibility(),
                estimated_bytes: 0,
                source: None,
                native_packages: Vec::new(),
                extra: None,
            });
        }
        ComponentKind::Config => {
            out.push(PlannedAction {
                id: component.id.clone(),
                kind: ActionKind::WriteConfig,
                target: component.id.clone(),
                display: component.display_name(),
                section: section.into(),
                profile: profile.into(),
                reason: component
                    .notes
                    .clone()
                    .unwrap_or_else(|| "write configuration".into()),
                already_satisfied: false,
                reversibility: component.reversibility(),
                estimated_bytes: 4096,
                source: None,
                native_packages: Vec::new(),
                extra: None,
            });
        }
        ComponentKind::ThirdParty | ComponentKind::Python | ComponentKind::Package => {
            for logical in &component.packages {
                if logical.is_empty() {
                    continue;
                }
                let source = source_for(logical);
                let kind = if source.is_some() || component.kind == ComponentKind::ThirdParty {
                    if source.is_some() {
                        ActionKind::ThirdPartyInstall
                    } else {
                        ActionKind::InstallPackage
                    }
                } else if component.kind == ComponentKind::Python {
                    ActionKind::ThirdPartyInstall
                } else {
                    ActionKind::InstallPackage
                };
                if source.is_some() {
                    out.push(PlannedAction {
                        id: format!("{logical}-repo"),
                        kind: ActionKind::AddRepository,
                        target: logical.clone(),
                        display: format!("Trusted source: {logical}"),
                        section: section.into(),
                        profile: profile.into(),
                        reason: "register verified third-party source".into(),
                        already_satisfied: logical_installed(logical),
                        reversibility: vpsforge_core::Reversibility::PartiallyReversible,
                        estimated_bytes: 0,
                        source: source.clone(),
                        native_packages: Vec::new(),
                        extra: None,
                    });
                }
                out.push(PlannedAction {
                    id: logical.clone(),
                    kind,
                    target: logical.clone(),
                    display: component.display_name(),
                    section: section.into(),
                    profile: profile.into(),
                    reason: component
                        .notes
                        .clone()
                        .unwrap_or_else(|| format!("required by {profile}/{section}")),
                    already_satisfied: logical_installed(logical),
                    reversibility: component.reversibility(),
                    estimated_bytes: estimated_bytes(logical),
                    source,
                    native_packages: native_packages(logical, host.pkg),
                    extra: None,
                });
            }
            if component.packages.is_empty() && component.kind == ComponentKind::Python {
                out.push(PlannedAction {
                    id: component.id.clone(),
                    kind: ActionKind::ThirdPartyInstall,
                    target: component.id.clone(),
                    display: component.display_name(),
                    section: section.into(),
                    profile: profile.into(),
                    reason: component.notes.clone().unwrap_or_default(),
                    already_satisfied: installed,
                    reversibility: component.reversibility(),
                    estimated_bytes: 200 * 1024 * 1024,
                    source: None,
                    native_packages: Vec::new(),
                    extra: None,
                });
            }
        }
    }

    for service in &component.services {
        out.push(PlannedAction {
            id: format!("svc-{service}"),
            kind: ActionKind::EnableService,
            target: service.clone(),
            display: format!("Enable {service}"),
            section: section.into(),
            profile: profile.into(),
            reason: "service required by profile".into(),
            already_satisfied: service_active(service),
            reversibility: vpsforge_core::Reversibility::Reversible,
            estimated_bytes: 0,
            source: None,
            native_packages: Vec::new(),
            extra: None,
        });
    }

    for port in &component.firewall_ports {
        firewall_ports.insert(port.clone());
        out.push(PlannedAction {
            id: format!("fw-{port}"),
            kind: ActionKind::OpenFirewall,
            target: port.clone(),
            display: format!("Allow {port}"),
            section: section.into(),
            profile: profile.into(),
            reason: "required by selected service".into(),
            already_satisfied: false,
            reversibility: vpsforge_core::Reversibility::Reversible,
            estimated_bytes: 0,
            source: None,
            native_packages: Vec::new(),
            extra: None,
        });
    }

    out
}

fn service_active(name: &str) -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn add_profile_warnings(host: &HostFacts, profiles: &[Profile], warnings: &mut Vec<PlanWarning>) {
    let wants_ai = profiles.iter().any(|p| p.id == "ai");
    if wants_ai {
        if host.gpu.is_none() {
            warnings.push(PlanWarning {
                level: WarningLevel::Medium,
                message: "GPU not detected — CUDA and NVIDIA Container Toolkit will be skipped."
                    .into(),
            });
        }
        if host.ram_gib() < 8.0 {
            warnings.push(PlanWarning {
                level: WarningLevel::High,
                message: "Local 7B+ models may exceed available memory.".into(),
            });
        }
        if host.ram_gib() < 4.0 {
            warnings.push(PlanWarning {
                level: WarningLevel::High,
                message: "RAM is very low for local inference. Prefer API-backed or tiny models."
                    .into(),
            });
        }
    }
    if !host.linux {
        warnings.push(PlanWarning {
            level: WarningLevel::High,
            message: "This host is not Linux. Plans can be previewed, but packages will not be installed."
                .into(),
        });
    }
}

fn plan_title(ids: &[String]) -> String {
    if ids.len() == 1 {
        format!("{} PROFILE ANALYSIS", ids[0].to_ascii_uppercase())
    } else {
        format!(
            "COMBINED PROFILE ANALYSIS ({})",
            ids.join(" + ").to_ascii_uppercase()
        )
    }
}

#[derive(Debug, Clone)]
pub struct CustomComponent {
    pub id: String,
    pub name: String,
    pub default: bool,
}

pub fn custom_catalog() -> Vec<CustomComponent> {
    [
        ("docker", "Docker", true),
        ("postgresql", "PostgreSQL", true),
        ("redis", "Redis", true),
        ("mongodb", "MongoDB", false),
        ("nginx", "Nginx", true),
        ("caddy", "Caddy", false),
        ("python3", "Python", true),
        ("nodejs", "Node.js", true),
        ("openjdk", "Java", false),
        ("golang", "Go", false),
        ("git", "Git", true),
        ("tmux", "tmux", true),
        ("fail2ban", "fail2ban", true),
    ]
    .into_iter()
    .map(|(id, name, default)| CustomComponent {
        id: id.into(),
        name: name.into(),
        default,
    })
    .collect()
}

pub fn build_custom_plan(host: &HostFacts, selected: &[String]) -> ForgeResult<InstallPlan> {
    let mut profile = Profile {
        id: "custom".into(),
        name: "Custom Build".into(),
        description: "User-selected components".into(),
        aliases: Vec::new(),
        sections: vec![vpsforge_profiles::ProfileSection {
            id: "selected".into(),
            title: "Selected".into(),
            depends_on: Vec::new(),
            condition: None,
            min_ram_mb: None,
            optional: false,
            default: true,
            flag: None,
            components: selected
                .iter()
                .map(|id| Component {
                    id: id.clone(),
                    name: Some(id.clone()),
                    kind: if source_for(id).is_some() {
                        ComponentKind::ThirdParty
                    } else {
                        ComponentKind::Package
                    },
                    packages: vec![id.clone()],
                    services: default_services(id),
                    firewall_ports: default_ports(id),
                    condition: None,
                    optional: false,
                    default: true,
                    flag: None,
                    notes: None,
                    reversibility: Default::default(),
                    tune: if id == "postgresql" {
                        Some("postgres".into())
                    } else {
                        None
                    },
                })
                .collect(),
        }],
    };
    // keep compiler happy if ProfileSection path differs
    let _ = &mut profile;
    build_plan(host, &[profile], &ProfileFlags::default())
}

fn default_services(id: &str) -> Vec<String> {
    match id {
        "docker" => vec!["docker".into()],
        "postgresql" => vec!["postgresql".into()],
        "redis" => vec!["redis".into()],
        "nginx" => vec!["nginx".into()],
        "caddy" => vec!["caddy".into()],
        "fail2ban" => vec!["fail2ban".into()],
        "mongodb" => vec!["mongod".into()],
        _ => Vec::new(),
    }
}

fn default_ports(id: &str) -> Vec<String> {
    match id {
        "nginx" | "caddy" => vec!["80/tcp".into(), "443/tcp".into()],
        "fail2ban" => vec!["22/tcp".into()],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vpsforge_detect::{constrained_host, sample_host};
    use vpsforge_profiles::get;

    #[test]
    fn tiny_box_skips_cuda() {
        let host = constrained_host();
        let profile = get("ai").unwrap();
        let plan = build_plan(&host, &[profile], &ProfileFlags::default()).unwrap();
        assert!(plan
            .skipped
            .iter()
            .any(|s| s.display.contains("NVIDIA") || s.id.contains("nvidia") || s.reason.contains("GPU")));
        assert!(plan.warnings.iter().any(|w| w.message.contains("7B")));
        assert!(plan
            .actions
            .iter()
            .any(|a| a.target == "python3" || a.id == "python3"));
    }

    #[test]
    fn gpu_box_includes_toolkit() {
        let host = sample_host();
        let profile = get("ai").unwrap();
        let plan = build_plan(&host, &[profile], &ProfileFlags::default()).unwrap();
        assert!(plan
            .actions
            .iter()
            .any(|a| a.target.contains("nvidia") || a.id.contains("nvidia")));
        assert!(plan.skipped.iter().any(|s| s.id == "pytorch-cpu"));
    }
}
