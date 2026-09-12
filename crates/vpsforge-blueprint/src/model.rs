use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    #[serde(default)]
    pub base: Base,
    #[serde(default)]
    pub requirements: Requirements,
    #[serde(default)]
    pub profiles: Vec<String>,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub services: Services,
    #[serde(default)]
    pub security: SecuritySpec,
    #[serde(default)]
    pub storage: StorageSpec,
    #[serde(default)]
    pub ai: AiSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base {
    #[serde(default = "debian")]
    pub family: String,
}

fn debian() -> String {
    "debian".into()
}

impl Default for Base {
    fn default() -> Self {
        Self { family: debian() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirements {
    #[serde(default = "default_mem")]
    pub memory: String,
    #[serde(default = "default_arches")]
    pub architecture: Vec<String>,
}

fn default_mem() -> String {
    ">=2GB".into()
}

fn default_arches() -> Vec<String> {
    vec!["amd64".into(), "arm64".into()]
}

impl Default for Requirements {
    fn default() -> Self {
        Self {
            memory: default_mem(),
            architecture: default_arches(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Services {
    #[serde(default)]
    pub docker: ServiceToggle,
    #[serde(default)]
    pub nginx: ServiceToggle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceToggle {
    #[serde(default)]
    pub enabled: bool,
}

impl Default for ServiceToggle {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    #[serde(default)]
    pub ssh: SshSpec,
    #[serde(default)]
    pub firewall: FirewallSpec,
}

impl Default for SecuritySpec {
    fn default() -> Self {
        Self {
            ssh: SshSpec::default(),
            firewall: FirewallSpec::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSpec {
    #[serde(default = "default_false")]
    pub password_authentication: bool,
    #[serde(default = "default_false")]
    pub root_login: bool,
}

fn default_false() -> bool {
    false
}

impl Default for SshSpec {
    fn default() -> Self {
        Self {
            password_authentication: false,
            root_login: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallSpec {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_ports")]
    pub ports: Vec<u16>,
}

fn default_true() -> bool {
    true
}

fn default_ports() -> Vec<u16> {
    vec![22, 80, 443]
}

impl Default for FirewallSpec {
    fn default() -> Self {
        Self {
            enabled: true,
            ports: default_ports(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageSpec {
    #[serde(default)]
    pub postgres: bool,
    #[serde(default)]
    pub redis: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSpec {
    #[serde(default)]
    pub ollama: bool,
    #[serde(default = "auto")]
    pub gpu: String,
}

fn auto() -> String {
    "auto".into()
}

impl Default for AiSpec {
    fn default() -> Self {
        Self {
            ollama: false,
            gpu: auto(),
        }
    }
}

impl Blueprint {
    pub fn template(name: &str) -> Self {
        Self {
            name: name.to_string(),
            base: Base::default(),
            requirements: Requirements::default(),
            profiles: vec!["production".into()],
            packages: vec!["git".into(), "tmux".into(), "jq".into()],
            services: Services {
                docker: ServiceToggle { enabled: true },
                nginx: ServiceToggle { enabled: true },
            },
            security: SecuritySpec::default(),
            storage: StorageSpec {
                postgres: true,
                redis: true,
            },
            ai: AiSpec {
                ollama: true,
                gpu: "auto".into(),
            },
        }
    }

    pub fn expanded_profiles(&self) -> Vec<String> {
        let mut ids = self.profiles.clone();
        if self.storage.postgres || self.storage.redis {
            if !ids.iter().any(|p| p == "storage") {
                ids.push("storage".into());
            }
        }
        if self.ai.ollama && !ids.iter().any(|p| p == "ai") {
            ids.push("ai".into());
        }
        ids
    }
}
