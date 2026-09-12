use serde::{Deserialize, Serialize};

use vpsforge_core::{HostFacts, Reversibility};

use crate::condition::Condition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub sections: Vec<ProfileSection>,
}

impl Profile {
    pub fn all_ids(&self) -> Vec<String> {
        let mut ids = vec![self.id.clone()];
        ids.extend(self.aliases.iter().cloned());
        ids
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub min_ram_mb: Option<u64>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default = "default_true")]
    pub default: bool,
    #[serde(default)]
    pub flag: Option<String>,
    #[serde(default)]
    pub components: Vec<Component>,
}

fn default_true() -> bool {
    true
}

impl ProfileSection {
    pub fn skip_reason(&self, host: &HostFacts) -> Option<String> {
        if let Some(min) = self.min_ram_mb {
            if host.ram_mib() < min {
                return Some(format!(
                    "requires at least {min} MB RAM (detected {} MB)",
                    host.ram_mib()
                ));
            }
        }
        self.condition
            .as_ref()
            .and_then(|c| c.skip_reason(host))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub kind: ComponentKind,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub firewall_ports: Vec<String>,
    #[serde(default)]
    pub condition: Option<Condition>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default = "default_true")]
    pub default: bool,
    #[serde(default)]
    pub flag: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub reversibility: ReversibilitySpec,
    #[serde(default)]
    pub tune: Option<String>,
}

impl Component {
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| title_case(&self.id))
    }

    pub fn skip_reason(&self, host: &HostFacts) -> Option<String> {
        self.condition
            .as_ref()
            .and_then(|c| c.skip_reason(host))
    }

    pub fn reversibility(&self) -> Reversibility {
        self.reversibility.into()
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    #[default]
    Package,
    ThirdParty,
    Python,
    Config,
    Check,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilitySpec {
    #[default]
    Reversible,
    PartiallyReversible,
    NonReversible,
}

impl From<ReversibilitySpec> for Reversibility {
    fn from(value: ReversibilitySpec) -> Self {
        match value {
            ReversibilitySpec::Reversible => Self::Reversible,
            ReversibilitySpec::PartiallyReversible => Self::PartiallyReversible,
            ReversibilitySpec::NonReversible => Self::NonReversible,
        }
    }
}

fn title_case(id: &str) -> String {
    id.split(['-', '_'])
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
