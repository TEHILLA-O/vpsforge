use std::path::{Path, PathBuf};

use crate::APP_NAME;

#[derive(Debug, Clone)]
pub struct ForgePaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub log_dir: PathBuf,
    pub blueprint_dir: PathBuf,
    pub checkpoint_dir: PathBuf,
    pub app_dir: PathBuf,
    pub history_db: PathBuf,
}

impl ForgePaths {
    pub fn discover() -> Self {
        let (config_dir, data_dir, state_dir, log_dir) = if cfg!(windows) {
            let base = dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(APP_NAME);
            let config = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(APP_NAME);
            (config, base.clone(), base.clone(), base.join("logs"))
        } else if is_root() {
            (
                PathBuf::from("/etc/vpsforge"),
                PathBuf::from("/var/lib/vpsforge"),
                PathBuf::from("/var/lib/vpsforge"),
                PathBuf::from("/var/log/vpsforge"),
            )
        } else {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let data = home.join(".local/share/vpsforge");
            (
                home.join(".config/vpsforge"),
                data.clone(),
                data.clone(),
                home.join(".local/state/vpsforge/logs"),
            )
        };

        let blueprint_dir = config_dir.join("blueprints");
        let checkpoint_dir = state_dir.join("checkpoints");
        let app_dir = config_dir.join("apps");
        let history_db = state_dir.join("history.db");

        Self {
            config_dir,
            data_dir,
            state_dir,
            log_dir,
            blueprint_dir,
            checkpoint_dir,
            app_dir,
            history_db,
        }
    }

    pub fn ensure(&self) -> std::io::Result<()> {
        for dir in [
            &self.config_dir,
            &self.data_dir,
            &self.state_dir,
            &self.log_dir,
            &self.blueprint_dir,
            &self.checkpoint_dir,
            &self.app_dir,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    pub fn blueprint_file(&self, name: &str) -> PathBuf {
        self.blueprint_dir.join(format!("{name}.yaml"))
    }

    pub fn app_root(&self, name: &str) -> PathBuf {
        self.app_dir.join(name)
    }

    pub fn profile_override_dir(&self) -> PathBuf {
        self.config_dir.join("profiles")
    }
}

pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        nix_uid() == 0
    }
    #[cfg(windows)]
    {
        false
    }
}

#[cfg(unix)]
fn nix_uid() -> u32 {
    // Avoid a hard nix dependency in core; use libc-less fallback.
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(1000)
}

pub fn exists_cmd(name: &str) -> bool {
    which_like(name).is_some()
}

pub fn which_like(name: &str) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(windows)]
            {
                for ext in ["exe", "bat", "cmd"] {
                    let with_ext = dir.join(format!("{name}.{ext}"));
                    if with_ext.is_file() {
                        return Some(with_ext);
                    }
                }
            }
        }
    }
    None
}

pub fn read_optional(path: impl AsRef<Path>) -> Option<String> {
    std::fs::read_to_string(path).ok()
}
