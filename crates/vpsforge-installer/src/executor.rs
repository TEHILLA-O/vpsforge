use std::process::Command;

use vpsforge_core::{
    ActionKind, ApplyMode, ApplyResult, ForgeError, ForgeResult, HostFacts, InstallPlan,
};
use vpsforge_packages::{command_for, verify_source};

#[derive(Debug, Clone)]
pub struct ApplyOptions {
    pub mode: ApplyMode,
    pub apply_tuning: bool,
    pub allow_unverified: bool,
}

impl Default for ApplyOptions {
    fn default() -> Self {
        Self {
            mode: ApplyMode::DryRun,
            apply_tuning: false,
            allow_unverified: false,
        }
    }
}

pub fn apply_plan(
    host: &HostFacts,
    plan: &InstallPlan,
    opts: &ApplyOptions,
) -> ForgeResult<ApplyResult> {
    if opts.mode == ApplyMode::DryRun {
        return Ok(ApplyResult {
            applied: Vec::new(),
            skipped: plan
                .actions
                .iter()
                .map(|a| format!("dry-run: {}", a.display))
                .collect(),
            failed: Vec::new(),
            checkpoint: None,
        });
    }

    if !host.supports_installs() {
        return Err(ForgeError::UnsupportedPlatform(
            "package installation requires a supported Linux distribution".into(),
        ));
    }

    let mgr = command_for(host.pkg);
    let mut result = ApplyResult {
        applied: Vec::new(),
        skipped: Vec::new(),
        failed: Vec::new(),
        checkpoint: None,
    };

    for action in &plan.actions {
        if action.already_satisfied {
            result
                .skipped
                .push(format!("{} already configured", action.display));
            continue;
        }

        if let Some(source) = &action.source {
            verify_source(source)?;
            if !source.has_integrity() && !opts.allow_unverified {
                return Err(ForgeError::Trust(format!(
                    "{} has no registered checksum or signing fingerprint. Re-run with --allow-unverified after reviewing the source.",
                    source.name
                )));
            }
        }

        let outcome = match action.kind {
            ActionKind::InstallPackage => {
                if action.native_packages.is_empty() {
                    Ok(())
                } else {
                    run_std(mgr.install_cmd(&action.native_packages).to_std())
                }
            }
            ActionKind::RemovePackage => run_std(mgr.remove_cmd(&action.native_packages).to_std()),
            ActionKind::EnableService => enable_service(&action.target),
            ActionKind::OpenFirewall => open_port(&action.target),
            ActionKind::AddRepository | ActionKind::ThirdPartyInstall => {
                // Never curl | bash. Prefer the distro package when mapped;
                // otherwise require a verified local artefact.
                if action.native_packages.is_empty() {
                    Err(ForgeError::Trust(format!(
                        "refusing to install {} via remote script; download the official package, verify its checksum, then place the binary on PATH",
                        action.display
                    )))
                } else {
                    run_std(mgr.install_cmd(&action.native_packages).to_std())
                }
            }
            ActionKind::WriteConfig | ActionKind::TuneConfig | ActionKind::CreateDirectory => Ok(()),
            ActionKind::RunCommand => Ok(()),
        };

        match outcome {
            Ok(()) => result.applied.push(action.display.clone()),
            Err(err) => result.failed.push(format!("{}: {err}", action.display)),
        }
    }

    let _ = opts.apply_tuning;
    Ok(result)
}

fn run_std(mut cmd: Command) -> ForgeResult<()> {
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(ForgeError::PackageManager(format!(
            "{} exited with {status}",
            cmd.get_program().to_string_lossy()
        )))
    }
}

fn enable_service(name: &str) -> ForgeResult<()> {
    let mut cmd = Command::new("sudo");
    cmd.args(["systemctl", "enable", "--now", name]);
    run_std(cmd)
}

fn open_port(port: &str) -> ForgeResult<()> {
    let mut cmd = Command::new("sudo");
    cmd.args(["ufw", "allow", port]);
    run_std(cmd)
}
