use vpsforge_core::{CheckStatus, DoctorCheck, DoctorReport, HostFacts};
use vpsforge_packages::logical_installed;

pub fn system_doctor(host: &HostFacts) -> DoctorReport {
    let mut checks = Vec::new();

    checks.push(DoctorCheck {
        group: "SYSTEM".into(),
        name: "Supported distribution".into(),
        status: if host.pkg.is_supported() {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        detail: format!("{} ({})", host.os.pretty_name, host.pkg.as_str()),
    });
    checks.push(DoctorCheck {
        group: "SYSTEM".into(),
        name: "Package manager healthy".into(),
        status: if host.pkg.is_supported() {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        detail: host.pkg.as_str().into(),
    });
    checks.push(DoctorCheck {
        group: "SYSTEM".into(),
        name: "DNS working".into(),
        status: if dns_ok() {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        detail: if dns_ok() {
            "resolves".into()
        } else {
            "could not resolve".into()
        },
    });
    checks.push(DoctorCheck {
        group: "SYSTEM".into(),
        name: "Internet access".into(),
        status: if host.network.public_ipv4 {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        detail: if host.network.public_ipv4 {
            "default route present".into()
        } else {
            "no default route detected".into()
        },
    });
    checks.push(DoctorCheck {
        group: "SYSTEM".into(),
        name: "Disk space healthy".into(),
        status: if host.disk_available_gib() > 2.0 {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        detail: format!("{:.1} GB free", host.disk_available_gib()),
    });

    push_pkg(&mut checks, "Docker", &host.software.docker, "docker");
    push_pkg(&mut checks, "PostgreSQL", &host.software.postgresql, "postgresql");
    push_pkg(&mut checks, "nginx", &host.software.nginx, "nginx");
    push_pkg(&mut checks, "Redis", &host.software.redis, "redis");

    if host.has_nvidia() {
        checks.push(DoctorCheck {
            group: "AI".into(),
            name: "NVIDIA runtime".into(),
            status: if logical_installed("nvidia-container-toolkit") {
                CheckStatus::Pass
            } else {
                CheckStatus::Warn
            },
            detail: host
                .gpu
                .as_ref()
                .map(|g| g.model.clone())
                .unwrap_or_else(|| "NVIDIA".into()),
        });
    }

    let mut report = DoctorReport {
        checks,
        overall: String::new(),
        critical: 0,
        warnings: 0,
    };
    report.summarize();
    report
}

pub fn ai_doctor(host: &HostFacts) -> DoctorReport {
    let mut checks = Vec::new();
    push_pkg(&mut checks, "Python", &host.software.python, "python3");
    checks.last_mut().unwrap().group = "AI ENVIRONMENT".into();
    push_pkg(&mut checks, "uv", &host.software.uv, "uv");
    push_pkg(&mut checks, "Docker", &host.software.docker, "docker");
    push_pkg(&mut checks, "Ollama", &host.software.ollama, "ollama");

    let cuda = host.gpu.as_ref().and_then(|g| g.cuda_version.clone());
    checks.push(DoctorCheck {
        group: "AI ENVIRONMENT".into(),
        name: "CUDA".into(),
        status: if host.cuda_capable() {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        detail: cuda.unwrap_or_else(|| {
            if host.cuda_capable() {
                "detected".into()
            } else {
                "not available".into()
            }
        }),
    });
    checks.push(DoctorCheck {
        group: "AI ENVIRONMENT".into(),
        name: "GPU".into(),
        status: if host.has_nvidia() {
            CheckStatus::Pass
        } else {
            CheckStatus::Warn
        },
        detail: host
            .gpu
            .as_ref()
            .map(|g| g.model.clone())
            .unwrap_or_else(|| "Not detected".into()),
    });
    checks.push(DoctorCheck {
        group: "AI ENVIRONMENT".into(),
        name: "Container GPU access".into(),
        status: if host.has_nvidia() && logical_installed("nvidia-container-toolkit") {
            CheckStatus::Pass
        } else if host.has_nvidia() {
            CheckStatus::Warn
        } else {
            CheckStatus::Skip
        },
        detail: if logical_installed("nvidia-container-toolkit") {
            "nvidia-container-toolkit present".into()
        } else {
            "toolkit not installed".into()
        },
    });
    checks.push(DoctorCheck {
        group: "AI ENVIRONMENT".into(),
        name: "PyTorch CUDA".into(),
        status: if host.cuda_capable() {
            CheckStatus::Warn
        } else {
            CheckStatus::Skip
        },
        detail: "runtime import not executed in doctor; install the CUDA wheel on GPU hosts".into(),
    });

    for check in &mut checks {
        if check.group != "AI ENVIRONMENT" {
            check.group = "AI ENVIRONMENT".into();
        }
    }

    let mut report = DoctorReport {
        checks,
        overall: String::new(),
        critical: 0,
        warnings: 0,
    };
    report.summarize();
    if report.critical == 0 && report.warnings == 0 {
        report.overall = "AI environment ready.".into();
    }
    report
}

fn push_pkg(
    checks: &mut Vec<DoctorCheck>,
    name: &str,
    status: &vpsforge_core::SoftwareStatus,
    group_or_logical: &str,
) {
    checks.push(DoctorCheck {
        group: "PACKAGES".into(),
        name: name.into(),
        status: if status.installed || logical_installed(group_or_logical) {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        detail: status.display(),
    });
}

fn dns_ok() -> bool {
    std::fs::read_to_string("/etc/resolv.conf")
        .map(|t| t.lines().any(|l| l.trim_start().starts_with("nameserver")))
        .unwrap_or(cfg!(windows))
}
