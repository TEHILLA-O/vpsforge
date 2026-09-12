use std::process::Command;

use vpsforge_core::{ForgeResult, PackageManagerKind};

use crate::{logical_installed, PackageManager, UpdateSummary};

#[derive(Debug, Clone)]
pub struct PackageCommand {
    pub program: String,
    pub args: Vec<String>,
    pub description: String,
}

impl PackageCommand {
    pub fn new(program: impl Into<String>, args: &[&str], description: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            description: description.into(),
        }
    }

    pub fn display(&self) -> String {
        let mut parts = vec![self.program.clone()];
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }

    pub fn to_std(&self) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args);
        cmd
    }
}

pub struct Apt;
pub struct Dnf;
pub struct Pacman;
pub struct Zypper;
pub struct Apk;
pub struct Unsupported;

impl PackageManager for Apt {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Apt
    }
    fn is_installed(&self, logical: &str) -> bool {
        logical_installed(logical)
    }
    fn install_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec![
            "apt-get".into(),
            "install".into(),
            "-y".into(),
            "--no-install-recommends".into(),
        ];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("install {}", native.join(" ")),
        }
    }
    fn remove_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["apt-get".into(), "remove".into(), "-y".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("remove {}", native.join(" ")),
        }
    }
    fn search_cmd(&self, query: &str) -> PackageCommand {
        PackageCommand::new("apt-cache", &["search", query], format!("search {query}"))
    }
    fn update_index_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["apt-get".into(), "update".into()],
            description: "refresh apt index".into(),
        }
    }
    fn upgrade_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["apt-get".into(), "upgrade".into(), "-y".into()],
            description: "upgrade packages".into(),
        }
    }
    fn pending_updates(&self) -> ForgeResult<UpdateSummary> {
        Ok(UpdateSummary::default())
    }
}

impl PackageManager for Dnf {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Dnf
    }
    fn is_installed(&self, logical: &str) -> bool {
        logical_installed(logical)
    }
    fn install_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["dnf".into(), "install".into(), "-y".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("install {}", native.join(" ")),
        }
    }
    fn remove_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["dnf".into(), "remove".into(), "-y".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("remove {}", native.join(" ")),
        }
    }
    fn search_cmd(&self, query: &str) -> PackageCommand {
        PackageCommand::new("dnf", &["search", query], format!("search {query}"))
    }
    fn update_index_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["dnf".into(), "makecache".into()],
            description: "refresh dnf cache".into(),
        }
    }
    fn upgrade_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["dnf".into(), "upgrade".into(), "-y".into()],
            description: "upgrade packages".into(),
        }
    }
    fn pending_updates(&self) -> ForgeResult<UpdateSummary> {
        Ok(UpdateSummary::default())
    }
}

impl PackageManager for Pacman {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Pacman
    }
    fn is_installed(&self, logical: &str) -> bool {
        logical_installed(logical)
    }
    fn install_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["pacman".into(), "-S".into(), "--noconfirm".into(), "--needed".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("install {}", native.join(" ")),
        }
    }
    fn remove_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["pacman".into(), "-R".into(), "--noconfirm".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("remove {}", native.join(" ")),
        }
    }
    fn search_cmd(&self, query: &str) -> PackageCommand {
        PackageCommand::new("pacman", &["-Ss", query], format!("search {query}"))
    }
    fn update_index_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["pacman".into(), "-Sy".into()],
            description: "refresh pacman sync".into(),
        }
    }
    fn upgrade_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["pacman".into(), "-Syu".into(), "--noconfirm".into()],
            description: "upgrade packages".into(),
        }
    }
    fn pending_updates(&self) -> ForgeResult<UpdateSummary> {
        Ok(UpdateSummary::default())
    }
}

impl PackageManager for Zypper {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Zypper
    }
    fn is_installed(&self, logical: &str) -> bool {
        logical_installed(logical)
    }
    fn install_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["zypper".into(), "--non-interactive".into(), "install".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("install {}", native.join(" ")),
        }
    }
    fn remove_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["zypper".into(), "--non-interactive".into(), "remove".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("remove {}", native.join(" ")),
        }
    }
    fn search_cmd(&self, query: &str) -> PackageCommand {
        PackageCommand::new("zypper", &["search", query], format!("search {query}"))
    }
    fn update_index_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["zypper".into(), "refresh".into()],
            description: "refresh zypper".into(),
        }
    }
    fn upgrade_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec![
                "zypper".into(),
                "--non-interactive".into(),
                "update".into(),
            ],
            description: "upgrade packages".into(),
        }
    }
    fn pending_updates(&self) -> ForgeResult<UpdateSummary> {
        Ok(UpdateSummary::default())
    }
}

impl PackageManager for Apk {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Apk
    }
    fn is_installed(&self, logical: &str) -> bool {
        logical_installed(logical)
    }
    fn install_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["apk".into(), "add".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("install {}", native.join(" ")),
        }
    }
    fn remove_cmd(&self, native: &[String]) -> PackageCommand {
        let mut args = vec!["apk".into(), "del".into()];
        args.extend(native.iter().cloned());
        PackageCommand {
            program: "sudo".into(),
            args,
            description: format!("remove {}", native.join(" ")),
        }
    }
    fn search_cmd(&self, query: &str) -> PackageCommand {
        PackageCommand::new("apk", &["search", query], format!("search {query}"))
    }
    fn update_index_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["apk".into(), "update".into()],
            description: "refresh apk".into(),
        }
    }
    fn upgrade_cmd(&self) -> PackageCommand {
        PackageCommand {
            program: "sudo".into(),
            args: vec!["apk".into(), "upgrade".into()],
            description: "upgrade packages".into(),
        }
    }
    fn pending_updates(&self) -> ForgeResult<UpdateSummary> {
        Ok(UpdateSummary::default())
    }
}

impl PackageManager for Unsupported {
    fn kind(&self) -> PackageManagerKind {
        PackageManagerKind::Unknown
    }
    fn is_installed(&self, logical: &str) -> bool {
        logical_installed(logical)
    }
    fn install_cmd(&self, native: &[String]) -> PackageCommand {
        PackageCommand::new("echo", &["unsupported"], native.join(" "))
    }
    fn remove_cmd(&self, native: &[String]) -> PackageCommand {
        PackageCommand::new("echo", &["unsupported"], native.join(" "))
    }
    fn search_cmd(&self, query: &str) -> PackageCommand {
        PackageCommand::new("echo", &[query], query)
    }
    fn update_index_cmd(&self) -> PackageCommand {
        PackageCommand::new("echo", &["unsupported"], "update")
    }
    fn upgrade_cmd(&self) -> PackageCommand {
        PackageCommand::new("echo", &["unsupported"], "upgrade")
    }
    fn pending_updates(&self) -> ForgeResult<UpdateSummary> {
        Ok(UpdateSummary::default())
    }
}

pub fn command_for(kind: PackageManagerKind) -> Box<dyn PackageManager> {
    crate::manager(kind)
}
