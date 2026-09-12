//! Profile loading, conditions, and dependency ordering.

mod condition;
mod graph;
mod model;

use std::collections::HashMap;

use vpsforge_core::{ForgeError, ForgeResult, HostFacts};

pub use condition::Condition;
pub use graph::{ordered_components, GraphNode};
pub use model::{Component, ComponentKind, Profile, ProfileSection, ReversibilitySpec};

pub const EMBEDDED_PROFILES: &[(&str, &str)] = &[
    ("ai", include_str!("../../../profiles/ai.yaml")),
    ("storage", include_str!("../../../profiles/storage.yaml")),
    ("server", include_str!("../../../profiles/server.yaml")),
    ("devops", include_str!("../../../profiles/devops.yaml")),
    ("security", include_str!("../../../profiles/security.yaml")),
    ("development", include_str!("../../../profiles/development.yaml")),
    ("production", include_str!("../../../profiles/production.yaml")),
    ("minimal", include_str!("../../../profiles/minimal.yaml")),
];

pub fn load_embedded() -> ForgeResult<Vec<Profile>> {
    EMBEDDED_PROFILES
        .iter()
        .map(|(id, yaml)| {
            parse_profile(yaml).map_err(|e| ForgeError::msg(format!("profile {id}: {e}")))
        })
        .collect()
}

pub fn parse_profile(yaml: &str) -> ForgeResult<Profile> {
    serde_yaml::from_str(yaml).map_err(|e| ForgeError::msg(format!("invalid profile yaml: {e}")))
}

pub fn catalog() -> ForgeResult<HashMap<String, Profile>> {
    let mut map = HashMap::new();
    for profile in load_embedded()? {
        for alias in profile.all_ids() {
            map.insert(alias, profile.clone());
        }
    }
    Ok(map)
}

pub fn get(id: &str) -> ForgeResult<Profile> {
    let key = normalize_profile_id(id);
    catalog()?
        .get(&key)
        .cloned()
        .ok_or_else(|| ForgeError::ProfileNotFound(id.to_string()))
}

pub fn normalize_profile_id(id: &str) -> String {
    match id.to_ascii_lowercase().as_str() {
        "ml" | "machine-learning" | "ai" => "ai".into(),
        "data" | "database" | "db" | "storage" => "storage".into(),
        "web" | "www" | "server" => "server".into(),
        "containers" | "k8s" | "devops" => "devops".into(),
        "sec" | "harden" | "security" => "security".into(),
        "dev" | "workstation" | "development" => "development".into(),
        "prod" | "production" => "production".into(),
        "minimal" | "min" => "minimal".into(),
        "custom" => "custom".into(),
        other => other.to_string(),
    }
}

pub fn resolve_many(ids: &[String]) -> ForgeResult<Vec<Profile>> {
    let mut seen = Vec::new();
    for id in ids {
        let profile = get(id)?;
        if !seen.iter().any(|p: &Profile| p.id == profile.id) {
            seen.push(profile);
        }
    }
    Ok(seen)
}

pub fn list_summaries() -> ForgeResult<Vec<(String, String)>> {
    Ok(load_embedded()?
        .into_iter()
        .map(|p| (p.id, p.description))
        .collect())
}

pub fn evaluate_profile(profile: &Profile, host: &HostFacts, flags: &ProfileFlags) -> EvaluatedProfile {
    let mut sections = Vec::new();
    for section in &profile.sections {
        if !flags.wants_section(section) {
            sections.push(EvaluatedSection {
                section: section.clone(),
                included: false,
                skip_reason: Some(format!("optional section '{}' not selected", section.title)),
                components: Vec::new(),
            });
            continue;
        }
        if let Some(reason) = section.skip_reason(host) {
            sections.push(EvaluatedSection {
                section: section.clone(),
                included: false,
                skip_reason: Some(reason),
                components: Vec::new(),
            });
            continue;
        }
        let mut components = Vec::new();
        for component in &section.components {
            if let Some(reason) = component.skip_reason(host) {
                components.push(EvaluatedComponent {
                    component: component.clone(),
                    included: false,
                    skip_reason: Some(reason),
                });
            } else {
                components.push(EvaluatedComponent {
                    component: component.clone(),
                    included: true,
                    skip_reason: None,
                });
            }
        }
        sections.push(EvaluatedSection {
            section: section.clone(),
            included: true,
            skip_reason: None,
            components,
        });
    }
    EvaluatedProfile {
        profile: profile.clone(),
        sections,
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfileFlags {
    pub postgres: Option<bool>,
    pub redis: Option<bool>,
    pub mariadb: Option<bool>,
    pub mongodb: Option<bool>,
    pub minio: Option<bool>,
    pub nginx: Option<bool>,
    pub caddy: Option<bool>,
    pub clamav: Option<bool>,
    pub crowdsec: Option<bool>,
    pub extra: Vec<String>,
}

impl ProfileFlags {
    pub fn wants_section(&self, section: &ProfileSection) -> bool {
        let flag = section.flag.as_deref().unwrap_or("");
        let explicit = match flag {
            "postgres" => self.postgres,
            "redis" => self.redis,
            "mariadb" => self.mariadb,
            "mongodb" => self.mongodb,
            "minio" => self.minio,
            "nginx" => self.nginx,
            "caddy" => self.caddy,
            "clamav" => self.clamav,
            "crowdsec" => self.crowdsec,
            other if !other.is_empty() => {
                if self.extra.iter().any(|e| e == other) {
                    Some(true)
                } else {
                    None
                }
            }
            _ => None,
        };
        match explicit {
            Some(v) => v,
            None => !section.optional || section.default,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvaluatedProfile {
    pub profile: Profile,
    pub sections: Vec<EvaluatedSection>,
}

#[derive(Debug, Clone)]
pub struct EvaluatedSection {
    pub section: ProfileSection,
    pub included: bool,
    pub skip_reason: Option<String>,
    pub components: Vec<EvaluatedComponent>,
}

#[derive(Debug, Clone)]
pub struct EvaluatedComponent {
    pub component: Component,
    pub included: bool,
    pub skip_reason: Option<String>,
}

impl EvaluatedProfile {
    pub fn included_components(&self) -> Vec<&Component> {
        self.sections
            .iter()
            .filter(|s| s.included)
            .flat_map(|s| {
                s.components
                    .iter()
                    .filter(|c| c.included)
                    .map(|c| &c.component)
            })
            .collect()
    }

    pub fn skipped_components(&self) -> Vec<(&Component, String)> {
        let mut out = Vec::new();
        for section in &self.sections {
            if let Some(reason) = &section.skip_reason {
                for component in &section.section.components {
                    out.push((component, reason.clone()));
                }
            } else {
                for component in &section.components {
                    if let Some(reason) = &component.skip_reason {
                        out.push((&component.component, reason.clone()));
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_embedded_profiles_parse() {
        let profiles = load_embedded().unwrap();
        assert_eq!(profiles.len(), 8);
        assert!(profiles.iter().any(|p| p.id == "ai"));
    }

    #[test]
    fn aliases_resolve() {
        assert_eq!(get("ml").unwrap().id, "ai");
        assert_eq!(get("web").unwrap().id, "server");
    }
}
