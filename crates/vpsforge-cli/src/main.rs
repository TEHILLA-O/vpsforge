mod render;

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use dialoguer::{Confirm, MultiSelect};
use tracing_subscriber::EnvFilter;
use vpsforge_blueprint::{self as blueprint};
use vpsforge_checkpoint::{self as checkpoint};
use vpsforge_core::{ApplyMode, ForgePaths, HostFacts};
use vpsforge_detect::detect;
use vpsforge_image::{export as export_blueprint, ExportFormat};
use vpsforge_installer::{
    ai_doctor, apply_plan, build_custom_plan, custom_catalog, plan_profiles, postgres_tuning,
    status_report, system_doctor, ApplyOptions,
};
use vpsforge_packages::{logical_installed, manager, native_packages, search_catalog};
use vpsforge_profiles::{get as get_profile, list_summaries, resolve_many, ProfileFlags};
use vpsforge_security::{audit_host, network_scan_summary};

#[derive(Debug, Parser)]
#[command(
    name = "vpsforge",
    about = "Adaptive Linux VPS bootstrapper, toolkit manager, and server profile builder",
    version,
    propagate_version = true
)]
struct Cli {
    /// Print machine-readable JSON where supported
    #[arg(long, global = true)]
    json: bool,

    /// Increase logging (repeat for more detail)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Detect OS, hardware, virtualisation, and installed software
    Detect,
    /// Run system health checks
    Doctor,
    /// Show a compact host + software status
    Status,
    /// Install one or more profiles
    Install(InstallArgs),
    /// Remove a logical component
    Remove {
        component: String,
        #[arg(long)]
        yes: bool,
    },
    /// Refresh package indexes
    Update,
    /// Upgrade installed packages
    Upgrade {
        #[arg(long)]
        yes: bool,
    },
    /// Package search / install / remove
    Package {
        #[command(subcommand)]
        action: PackageCmd,
    },
    /// List, show, or apply profiles
    Profile {
        #[command(subcommand)]
        action: ProfileCmd,
    },
    /// Create and export reproducible blueprints
    Blueprint {
        #[command(subcommand)]
        action: BlueprintCmd,
    },
    /// Record a rollback checkpoint
    Checkpoint {
        #[arg(long)]
        note: Option<String>,
    },
    /// Roll back to a checkpoint
    Rollback { id: String },
    /// Defensive security tools
    Security {
        #[command(subcommand)]
        action: SecurityCmd,
    },
    /// Local listening-port observation
    Network {
        #[command(subcommand)]
        action: NetworkCmd,
    },
    /// List known services from the last plan
    Service {
        #[command(subcommand)]
        action: ServiceCmd,
    },
    /// Generate a Packer/cloud-init/shell image definition
    Image {
        #[command(subcommand)]
        action: ImageCmd,
    },
    /// Show VPSForge logs
    Logs,
    /// Show command history
    History {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// AI environment doctor
    Ai {
        #[command(subcommand)]
        action: AiCmd,
    },
    /// Generate an application layout
    Server {
        #[command(subcommand)]
        action: ServerCmd,
    },
    /// Generate shell completions
    Completions {
        shell: Shell,
    },
}

#[derive(Debug, Parser)]
struct InstallArgs {
    /// Profile ids: ai storage server devops security development production custom
    profiles: Vec<String>,
    #[arg(long)]
    postgres: bool,
    #[arg(long)]
    redis: bool,
    #[arg(long)]
    mariadb: bool,
    #[arg(long)]
    mongodb: bool,
    #[arg(long)]
    minio: bool,
    #[arg(long)]
    nginx: bool,
    #[arg(long)]
    caddy: bool,
    #[arg(long)]
    clamav: bool,
    #[arg(long)]
    crowdsec: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    apply_tuning: bool,
    #[arg(long)]
    allow_unverified: bool,
    #[arg(long)]
    no_checkpoint: bool,
}

#[derive(Debug, Subcommand)]
enum PackageCmd {
    Search { query: String },
    Install { name: String },
    Remove { name: String },
}

#[derive(Debug, Subcommand)]
enum ProfileCmd {
    List,
    Show { id: String },
    Apply {
        id: String,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
enum BlueprintCmd {
    Create {
        name: String,
        #[arg(long)]
        profiles: Vec<String>,
    },
    Edit { name: String },
    Validate { name: String },
    Apply {
        name: String,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        dry_run: bool,
    },
    Export {
        name: String,
        #[arg(long, default_value = "cloud-init")]
        format: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    List,
}

#[derive(Debug, Subcommand)]
enum SecurityCmd {
    Audit,
}

#[derive(Debug, Subcommand)]
enum NetworkCmd {
    Scan,
}

#[derive(Debug, Subcommand)]
enum ServiceCmd {
    List,
}

#[derive(Debug, Subcommand)]
enum ImageCmd {
    Build {
        name: String,
        #[arg(long, default_value = "packer")]
        format: String,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum AiCmd {
    Doctor,
}

#[derive(Debug, Subcommand)]
enum ServerCmd {
    Create { name: String },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("vpsforge: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    let host = detect();
    let paths = ForgePaths::discover();

    match cli.command {
        None => cmd_default(&host)?,
        Some(Commands::Detect) => {
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&host)?);
            } else {
                print!("{}", render::detect_report(&host));
            }
        }
        Some(Commands::Doctor) => {
            let report = system_doctor(&host);
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                render::print_doctor(&report);
            }
        }
        Some(Commands::Status) => {
            println!("{}", status_report(&host));
        }
        Some(Commands::Install(args)) => cmd_install(&host, &paths, args)?,
        Some(Commands::Remove { component, yes }) => cmd_remove(&host, &component, yes)?,
        Some(Commands::Update) => {
            let mgr = manager(host.pkg);
            println!("{}", mgr.update_index_cmd().display());
            if host.supports_installs() {
                let status = mgr.update_index_cmd().to_std().status()?;
                if !status.success() {
                    bail!("update failed");
                }
            }
        }
        Some(Commands::Upgrade { yes }) => {
            let mgr = manager(host.pkg);
            println!("{}", mgr.upgrade_cmd().display());
            if yes && host.supports_installs() {
                let status = mgr.upgrade_cmd().to_std().status()?;
                if !status.success() {
                    bail!("upgrade failed");
                }
            }
        }
        Some(Commands::Package { action }) => cmd_package(&host, action)?,
        Some(Commands::Profile { action }) => cmd_profile(&host, &paths, action)?,
        Some(Commands::Blueprint { action }) => cmd_blueprint(&host, &paths, action)?,
        Some(Commands::Checkpoint { note }) => {
            let cp = checkpoint::create_checkpoint(
                &paths,
                None,
                note.as_deref().unwrap_or("manual"),
            )?;
            println!("Creating VPSForge checkpoint...");
            println!();
            println!("✓ package state recorded");
            println!("✓ configuration backed up");
            println!("✓ service state recorded");
            println!("✓ firewall state recorded");
            println!();
            println!("Checkpoint:");
            println!();
            println!("{}", cp.id);
        }
        Some(Commands::Rollback { id }) => {
            let cp = checkpoint::get(&paths, &id)?;
            println!("{}", checkpoint::rollback_plan(&cp));
        }
        Some(Commands::Security { action: SecurityCmd::Audit }) => {
            let report = audit_host(&host);
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", report.render());
            }
        }
        Some(Commands::Network { action: NetworkCmd::Scan }) => {
            println!("{}", network_scan_summary(&host));
        }
        Some(Commands::Service { action: ServiceCmd::List }) => {
            println!("Known profile services: docker, nginx, postgresql, redis, fail2ban, ufw, ollama");
        }
        Some(Commands::Image { action }) => cmd_image(&paths, action)?,
        Some(Commands::Logs) => {
            let file = checkpoint::log_file(&paths);
            if file.exists() {
                print!("{}", std::fs::read_to_string(file)?);
            } else {
                println!("No log file yet at {}", file.display());
            }
        }
        Some(Commands::History { limit }) => {
            for event in checkpoint::history(&paths, limit)? {
                println!("{}  {}  {}", event.ts, event.command, event.detail);
            }
        }
        Some(Commands::Ai { action: AiCmd::Doctor }) => {
            render::print_doctor(&ai_doctor(&host));
        }
        Some(Commands::Server { action: ServerCmd::Create { name } }) => {
            cmd_server_create(&paths, &name)?;
        }
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "vpsforge", &mut io::stdout());
        }
    }
    Ok(())
}

fn init_tracing(verbose: u8) {
    let filter = match verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .with_target(false)
        .try_init();
}

fn cmd_default(host: &HostFacts) -> Result<()> {
    if !io::IsTerminal::is_terminal(&io::stdout()) {
        println!("{}", render::detect_report(host));
        return Ok(());
    }
    match vpsforge_tui::run(host)? {
        Some(outcome) if outcome.apply => {
            println!(
                "Selected {}. Re-run: vpsforge install {}",
                outcome.selected.join(" "),
                outcome.selected.join(" ")
            );
        }
        _ => {}
    }
    Ok(())
}

fn cmd_install(host: &HostFacts, paths: &ForgePaths, args: InstallArgs) -> Result<()> {
    let flags = ProfileFlags {
        postgres: flag(args.postgres),
        redis: flag(args.redis),
        mariadb: flag(args.mariadb),
        mongodb: flag(args.mongodb),
        minio: flag(args.minio),
        nginx: flag(args.nginx),
        caddy: flag(args.caddy),
        clamav: flag(args.clamav),
        crowdsec: flag(args.crowdsec),
        extra: Vec::new(),
    };

    let plan = if args.profiles.iter().any(|p| p == "custom") {
        let selected = interactive_custom()?;
        build_custom_plan(host, &selected)?
    } else if args.profiles.is_empty() {
        bail!("specify a profile, e.g. vpsforge install ai");
    } else {
        let profiles = resolve_many(&args.profiles)?;
        plan_profiles(host, &profiles, &flags)?
    };

    checkpoint::record(
        paths,
        "install",
        &format!("plan {}", plan.profiles.join(",")),
    )
    .ok();

    if plan.is_noop() {
        render::print_idempotent(&plan);
        return Ok(());
    }

    render::print_plan(&plan, host);
    println!();
    render::print_install_preview(&plan);

    if args.dry_run {
        return Ok(());
    }

    if !args.yes {
        println!();
        print!("Apply? [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if !matches!(line.trim(), "y" | "Y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let mut checkpoint_id = None;
    if !args.no_checkpoint {
        println!();
        println!("Creating VPSForge checkpoint...");
        let cp = checkpoint::create_checkpoint(paths, Some(&plan), "pre-install")?;
        println!("✓ package state recorded");
        println!("✓ configuration backed up");
        println!("✓ service state recorded");
        println!("✓ firewall state recorded");
        println!();
        println!("Checkpoint:");
        println!();
        println!("{}", cp.id);
        checkpoint_id = Some(cp.id);
    }

    let mut result = apply_plan(
        host,
        &plan,
        &ApplyOptions {
            mode: ApplyMode::Apply,
            apply_tuning: args.apply_tuning,
            allow_unverified: args.allow_unverified,
        },
    )?;
    result.checkpoint = checkpoint_id;
    println!();
    render::print_apply(&result);
    if !result.failed.is_empty() {
        bail!("one or more actions failed");
    }
    Ok(())
}

fn flag(set: bool) -> Option<bool> {
    if set {
        Some(true)
    } else {
        None
    }
}

fn interactive_custom() -> Result<Vec<String>> {
    let catalog = custom_catalog();
    let items: Vec<String> = catalog.iter().map(|c| c.name.clone()).collect();
    let defaults: Vec<bool> = catalog.iter().map(|c| c.default).collect();
    println!("What would you like to install?\n");
    let chosen = MultiSelect::new()
        .items(&items)
        .defaults(&defaults)
        .interact()?;
    Ok(chosen.into_iter().map(|i| catalog[i].id.clone()).collect())
}

fn cmd_remove(host: &HostFacts, component: &str, yes: bool) -> Result<()> {
    let native = native_packages(component, host.pkg);
    println!("Remove {component} => {}", native.join(" "));
    if !yes {
        if !Confirm::new()
            .with_prompt("Proceed?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }
    if !host.supports_installs() {
        bail!("package removal requires a supported Linux host");
    }
    let mgr = manager(host.pkg);
    let status = mgr.remove_cmd(&native).to_std().status()?;
    if !status.success() {
        bail!("remove failed");
    }
    Ok(())
}

fn cmd_package(host: &HostFacts, action: PackageCmd) -> Result<()> {
    match action {
        PackageCmd::Search { query } => {
            for pkg in search_catalog(&query) {
                println!("{:<28} {}", pkg.id, pkg.description);
            }
        }
        PackageCmd::Install { name } => {
            if logical_installed(&name) {
                println!("✓ {name} already installed");
                return Ok(());
            }
            let native = native_packages(&name, host.pkg);
            println!("Would install {} via {}", native.join(" "), host.pkg.as_str());
            if host.supports_installs() {
                let mgr = manager(host.pkg);
                let status = mgr.install_cmd(&native).to_std().status()?;
                if !status.success() {
                    bail!("install failed");
                }
            }
        }
        PackageCmd::Remove { name } => cmd_remove(host, &name, true)?,
    }
    Ok(())
}

fn cmd_profile(host: &HostFacts, paths: &ForgePaths, action: ProfileCmd) -> Result<()> {
    match action {
        ProfileCmd::List => {
            println!("{}", render::profile_table(&list_summaries()?));
        }
        ProfileCmd::Show { id } => {
            let profile = get_profile(&id)?;
            print!("{}", render::show_profile(&profile));
        }
        ProfileCmd::Apply { id, yes, dry_run } => {
            cmd_install(
                host,
                paths,
                InstallArgs {
                    profiles: vec![id],
                    postgres: false,
                    redis: false,
                    mariadb: false,
                    mongodb: false,
                    minio: false,
                    nginx: false,
                    caddy: false,
                    clamav: false,
                    crowdsec: false,
                    dry_run,
                    yes,
                    apply_tuning: false,
                    allow_unverified: false,
                    no_checkpoint: dry_run,
                },
            )?;
        }
    }
    Ok(())
}

fn cmd_blueprint(host: &HostFacts, paths: &ForgePaths, action: BlueprintCmd) -> Result<()> {
    match action {
        BlueprintCmd::Create { name, profiles } => {
            let bp = blueprint::create(&name, &profiles, host)?;
            let path = blueprint::save(paths, &bp)?;
            println!("{}", blueprint::render(&bp));
            println!("Wrote {}", path.display());
        }
        BlueprintCmd::Edit { name } => {
            let path = paths.blueprint_file(&name);
            if !path.exists() {
                let bp = blueprint::create(&name, &[], host)?;
                blueprint::save(paths, &bp)?;
            }
            println!("Blueprint file: {}", path.display());
            println!("Edit the YAML, then: vpsforge blueprint validate {name}");
        }
        BlueprintCmd::Validate { name } => {
            let bp = blueprint::load(paths, &name)?;
            println!("Blueprint `{name}` is valid ({} profiles)", bp.profiles.len());
        }
        BlueprintCmd::Apply {
            name,
            yes,
            dry_run,
        } => {
            let bp = blueprint::load(paths, &name)?;
            cmd_install(
                host,
                paths,
                InstallArgs {
                    profiles: bp.expanded_profiles(),
                    postgres: bp.storage.postgres,
                    redis: bp.storage.redis,
                    mariadb: false,
                    mongodb: false,
                    minio: false,
                    nginx: bp.services.nginx.enabled,
                    caddy: false,
                    clamav: false,
                    crowdsec: false,
                    dry_run,
                    yes,
                    apply_tuning: false,
                    allow_unverified: false,
                    no_checkpoint: dry_run,
                },
            )?;
        }
        BlueprintCmd::Export {
            name,
            format,
            output,
        } => {
            let bp = blueprint::load(paths, &name)?;
            let fmt = ExportFormat::parse(&format)
                .with_context(|| format!("unknown format {format}"))?;
            let body = export_blueprint(&bp, fmt);
            if let Some(path) = output {
                std::fs::write(&path, &body)?;
                println!("Wrote {}", path.display());
            } else {
                print!("{body}");
            }
        }
        BlueprintCmd::List => {
            for name in blueprint::list(paths)? {
                println!("{name}");
            }
        }
    }
    Ok(())
}

fn cmd_image(paths: &ForgePaths, action: ImageCmd) -> Result<()> {
    match action {
        ImageCmd::Build {
            name,
            format,
            output,
        } => {
            let bp = blueprint::load(paths, &name)?;
            let fmt = ExportFormat::parse(&format).unwrap_or(ExportFormat::Packer);
            let body = export_blueprint(&bp, fmt);
            let dest = output.unwrap_or_else(|| {
                PathBuf::from(format!("{name}.{}", fmt.extension()))
            });
            std::fs::write(&dest, body)?;
            println!("Wrote Packer/image definition to {}", dest.display());
            println!("This generates a template, not a running hypervisor image.");
        }
    }
    Ok(())
}

fn cmd_server_create(paths: &ForgePaths, name: &str) -> Result<()> {
    paths.ensure()?;
    let root = paths.app_root(name);
    std::fs::create_dir_all(root.join("systemd"))?;
    std::fs::create_dir_all(root.join("backup"))?;
    std::fs::write(
        root.join("app.yaml"),
        format!("name: {name}\nruntime: docker\n"),
    )?;
    std::fs::write(root.join(".env"), "APP_ENV=production\n")?;
    std::fs::write(
        root.join("docker-compose.yml"),
        format!(
            "services:\n  {name}:\n    image: {name}:latest\n    restart: unless-stopped\n    ports:\n      - \"8080:8080\"\n"
        ),
    )?;
    std::fs::write(
        root.join("nginx.conf"),
        format!(
            "server {{\n    listen 80;\n    server_name {name};\n    location / {{\n        proxy_pass http://127.0.0.1:8080;\n    }}\n}}\n"
        ),
    )?;
    println!("Created {}", root.display());
    println!("  app.yaml");
    println!("  .env");
    println!("  docker-compose.yml");
    println!("  nginx.conf");
    println!("  systemd/");
    println!("  backup/");
    let _ = postgres_tuning(16 * 1024 * 1024 * 1024);
    Ok(())
}
