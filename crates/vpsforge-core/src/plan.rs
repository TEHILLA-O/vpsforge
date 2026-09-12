use serde::{Deserialize, Serialize};

use crate::trust::TrustedSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    Reversible,
    PartiallyReversible,
    NonReversible,
}

impl Reversibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reversible => "reversible",
            Self::PartiallyReversible => "partially reversible",
            Self::NonReversible => "non-reversible",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    InstallPackage,
    AddRepository,
    EnableService,
    WriteConfig,
    OpenFirewall,
    ThirdPartyInstall,
    TuneConfig,
    CreateDirectory,
    RunCommand,
    RemovePackage,
}

impl ActionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InstallPackage => "install package",
            Self::AddRepository => "add repository",
            Self::EnableService => "enable service",
            Self::WriteConfig => "write config",
            Self::OpenFirewall => "open firewall",
            Self::ThirdPartyInstall => "third-party install",
            Self::TuneConfig => "tune config",
            Self::CreateDirectory => "create directory",
            Self::RunCommand => "run command",
            Self::RemovePackage => "remove package",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAction {
    pub id: String,
    pub kind: ActionKind,
    pub target: String,
    pub display: String,
    pub section: String,
    pub profile: String,
    pub reason: String,
    pub already_satisfied: bool,
    pub reversibility: Reversibility,
    pub estimated_bytes: u64,
    pub source: Option<TrustedSource>,
    pub native_packages: Vec<String>,
    pub extra: Option<String>,
}

impl PlannedAction {
    pub fn pending(&self) -> bool {
        !self.already_satisfied
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedAction {
    pub id: String,
    pub display: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WarningLevel {
    Info,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanWarning {
    pub level: WarningLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPlan {
    pub profiles: Vec<String>,
    pub title: String,
    pub actions: Vec<PlannedAction>,
    pub skipped: Vec<SkippedAction>,
    pub warnings: Vec<PlanWarning>,
    pub recommendations: Vec<String>,
    pub estimated_bytes: u64,
    pub repos: usize,
    pub services: usize,
    pub firewall_ports: Vec<String>,
    pub config_files: usize,
}

impl InstallPlan {
    pub fn pending_actions(&self) -> impl Iterator<Item = &PlannedAction> {
        self.actions.iter().filter(|a| a.pending())
    }

    pub fn pending_count(&self) -> usize {
        self.pending_actions().count()
    }

    pub fn already_count(&self) -> usize {
        self.actions.iter().filter(|a| a.already_satisfied).count()
    }

    pub fn package_count(&self) -> usize {
        self.pending_actions()
            .filter(|a| matches!(a.kind, ActionKind::InstallPackage | ActionKind::ThirdPartyInstall))
            .count()
    }

    pub fn is_noop(&self) -> bool {
        self.pending_count() == 0
    }

    pub fn estimated_gib(&self) -> f64 {
        self.estimated_bytes as f64 / 1024.0 / 1024.0 / 1024.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

impl CheckStatus {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Pass => "✓",
            Self::Warn => "⚠",
            Self::Fail => "✗",
            Self::Skip => "–",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub group: String,
    pub name: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
    pub overall: String,
    pub critical: usize,
    pub warnings: usize,
}

impl DoctorReport {
    pub fn summarize(&mut self) {
        self.critical = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Fail)
            .count();
        self.warnings = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Warn)
            .count();
        self.overall = if self.critical > 0 {
            "DEGRADED".into()
        } else if self.warnings > 0 {
            "HEALTHY WITH WARNINGS".into()
        } else {
            "HEALTHY".into()
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyMode {
    DryRun,
    Apply,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub applied: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
    pub checkpoint: Option<String>,
}
