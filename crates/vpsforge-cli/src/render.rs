use colored::Colorize;
use comfy_table::{presets::NOTHING, Table};
use vpsforge_core::{
    format, ActionKind, ApplyResult, CheckStatus, DoctorReport, HostFacts, InstallPlan,
};

pub fn detect_report(host: &HostFacts) -> String {
    let gpu = host
        .gpu
        .as_ref()
        .map(|g| g.model.clone())
        .unwrap_or_else(|| "Not detected".into());
    let cuda = if host.cuda_capable() { "Yes" } else { "No" };
    format!(
        "{title}

Operating System
{rule}
Distribution     {distro}
Version          {version}
Family           {family}

System
{rule}
Architecture     {arch}
Kernel           {kernel}
Init             {init}
Package Manager  {pkg}

Resources
{rule}
CPU              {cpu} cores
RAM              {ram}
Available RAM    {ram_avail}
Disk             {disk}
Available        {disk_avail}

Virtualisation
{rule}
Type             {virt}
Environment      {env}

Hardware
{rule}
GPU              {gpu}
CUDA capable     {cuda}

Network
{rule}
Public IPv4      {ipv4}
IPv6             {ipv6}

Current Software
{rule}
Docker           {docker}
Python           {python}
Node             {node}
PostgreSQL       {postgres}
Nginx            {nginx}
",
        title = "VPSFORGE SYSTEM DETECTION".bold(),
        rule = format::rule(28),
        distro = host.os.distribution,
        version = host.os.version,
        family = host.os.family.as_str(),
        arch = host.arch,
        kernel = host.kernel,
        init = host.init.as_str(),
        pkg = host.pkg.as_str(),
        cpu = host.cpu_cores,
        ram = format::gib(host.ram_bytes),
        ram_avail = format::gib(host.ram_available),
        disk = format::gib(host.disk_bytes),
        disk_avail = format::gib(host.disk_available),
        virt = host.virt.kind.as_str(),
        env = host.virt.environment,
        ipv4 = if host.network.public_ipv4 {
            "Detected"
        } else {
            "Not detected"
        },
        ipv6 = if host.network.ipv6 {
            "Available"
        } else {
            "Not detected"
        },
        docker = host.software.docker.display(),
        python = host.software.python.display(),
        node = host.software.node.display(),
        postgres = host.software.postgresql.display(),
        nginx = host.software.nginx.display(),
    )
}

pub fn print_plan(plan: &InstallPlan, host: &HostFacts) {
    println!("{}", plan.title.bold());
    println!();
    println!("GPU                {}", gpu_line(host));
    println!("RAM                {}", format::gib(host.ram_bytes));
    println!("Disk               {}", format::gib(host.disk_bytes));
    println!();
    println!("{}", "Recommended:".green().bold());
    for action in plan.actions.iter().filter(|a| {
        matches!(
            a.kind,
            ActionKind::InstallPackage | ActionKind::ThirdPartyInstall | ActionKind::WriteConfig
        )
    }) {
        let mark = if action.already_satisfied {
            "✓".green()
        } else {
            "+".yellow()
        };
        println!("{mark} {}", action.display);
    }
    if !plan.skipped.is_empty() {
        println!();
        println!("{}", "Skipped:".yellow().bold());
        for skip in &plan.skipped {
            println!("- {} ({})", skip.display, skip.reason);
        }
    }
    if !plan.warnings.is_empty() {
        println!();
        println!("{}", "Warning:".red().bold());
        for warn in &plan.warnings {
            println!("{}", warn.message);
        }
    }
    if !plan.recommendations.is_empty() {
        println!();
        for rec in &plan.recommendations {
            println!("{rec}");
        }
    }
}

pub fn print_install_preview(plan: &InstallPlan) {
    println!("{}", "INSTALLATION PLAN".bold());
    println!();
    println!("Install                         {:>2} packages", plan.package_count());
    println!("Add repositories                {:>2}", plan.repos);
    println!("Create services                 {:>2}", plan.services);
    println!(
        "Open firewall ports             {:>2}",
        plan.firewall_ports.len()
    );
    println!("Create configuration files     {:>3}", plan.config_files);
    println!(
        "Estimated disk use           {:>6}",
        format::bytes_human(plan.estimated_bytes)
    );
    println!();
    println!("{}", "Changes:".bold());
    println!();
    for action in plan.pending_actions().filter(|a| {
        matches!(
            a.kind,
            ActionKind::InstallPackage | ActionKind::ThirdPartyInstall
        )
    }) {
        println!("+ {}", action.display);
    }
    if !plan.firewall_ports.is_empty() {
        println!();
        println!("{}", "Firewall:".bold());
        println!();
        for port in &plan.firewall_ports {
            println!("+ {port}");
        }
    }
    if let Some(source) = plan.actions.iter().find_map(|a| a.source.as_ref()) {
        println!();
        println!("{}", "Third-party sources (never curl | bash):".bold());
        println!("  {} — {}", source.name, source.homepage);
        if let Some(fp) = &source.fingerprint {
            println!("  fingerprint {fp}");
        }
    }
    println!();
    println!("{}", "No changes have been made.".italic());
}

pub fn print_idempotent(plan: &InstallPlan) {
    println!("{}", format!("{} PROFILE", plan.profiles.join(" + ").to_ascii_uppercase()).bold());
    println!();
    for action in &plan.actions {
        if action.already_satisfied {
            println!("{} {} already configured", "✓".green(), action.display);
        }
    }
    if plan.is_noop() {
        println!();
        println!("{}", "No changes required.".green().bold());
    }
}

pub fn print_doctor(report: &DoctorReport) {
    println!("{}", "VPSFORGE DOCTOR".bold());
    println!();
    let mut current = String::new();
    for check in &report.checks {
        if check.group != current {
            if !current.is_empty() {
                println!();
            }
            println!("{}", check.group.bold());
            current = check.group.clone();
        }
        let glyph = match check.status {
            CheckStatus::Pass => "✓".green(),
            CheckStatus::Warn => "⚠".yellow(),
            CheckStatus::Fail => "✗".red(),
            CheckStatus::Skip => "–".normal(),
        };
        println!("{glyph} {:<22} {}", check.name, check.detail);
    }
    println!();
    println!("{}", "Overall".bold());
    println!("{}", format::rule(20));
    println!("Status: {}", report.overall.bold());
    println!();
    println!("Critical issues: {}", report.critical);
    println!("Warnings:        {}", report.warnings);
}

pub fn print_apply(result: &ApplyResult) {
    for item in &result.applied {
        println!("{} {item}", "✓".green());
    }
    for item in &result.skipped {
        println!("{} {item}", "·".normal());
    }
    for item in &result.failed {
        println!("{} {item}", "✗".red());
    }
    if let Some(cp) = &result.checkpoint {
        println!();
        println!("Checkpoint: {cp}");
    }
}

pub fn profile_table(rows: &[(String, String)]) -> String {
    let mut table = Table::new();
    table.load_preset(NOTHING);
    table.set_header(vec!["PROFILE", "DESCRIPTION"]);
    for (id, desc) in rows {
        table.add_row(vec![id, desc]);
    }
    table.to_string()
}

pub fn show_profile(profile: &vpsforge_profiles::Profile) -> String {
    let mut out = vec![profile.name.to_ascii_uppercase(), String::new()];
    for section in &profile.sections {
        let cond = if section.condition.is_some() || section.min_ram_mb.is_some() {
            " [conditional]"
        } else {
            ""
        };
        out.push(format!("{}{cond}", section.title));
        for component in &section.components {
            out.push(format!("    {}", component.display_name()));
        }
        out.push(String::new());
    }
    out.join("\n")
}

fn gpu_line(host: &HostFacts) -> String {
    host.gpu
        .as_ref()
        .map(|g| g.model.clone())
        .unwrap_or_else(|| "Not detected".into())
}
