# VPSForge

**Adaptive Linux VPS bootstrapper, toolkit manager, and server profile builder.**

Install one small binary on a fresh Linux VPS. VPSForge detects the machine, then turns it into an AI server, database host, web server, DevOps box, defensive security monitor, development workstation, or a blueprint you define yourself.

It is not a wrapper around `apt install`. It builds a **dependency graph**, evaluates **hardware conditions**, shows a **dry-run plan**, stays **idempotent**, records **checkpoints**, and can export the result as **cloud-init**, a **shell bootstrap**, or a **Packer** golden-image template.

```
vpsforge install ai
```

On a 2 CPU / 2 GB / no-GPU machine that does **not** dump a CUDA stack onto the disk. It recommends Python, uv, Git, Docker, CPU PyTorch, and a lightweight Ollama configuration — and it says why CUDA was skipped.

## Features

- Automatic OS, CPU, RAM, disk, GPU, virt, network, and software detection
- Composable profiles (`ai`, `storage`, `server`, `devops`, `security`, `development`, `production`, `minimal`)
- Conditional sections (NVIDIA stack only when an NVIDIA GPU exists)
- Package-manager abstraction: Apt, Dnf, Pacman, Zypper, Apk
- Installation preview before anything is touched
- Idempotent re-runs
- Checkpoints and rollback records
- Defensive security audit with a scored report
- Reproducible **blueprints**
- Export to cloud-init, shell, Packer HCL, and documentation
- Interactive TUI and a full non-interactive CLI
- No silent `curl | bash`. Third-party sources are listed, HTTPS-checked, and require checksum/signature data where available

## Install

Build from source:

```bash
cargo install --path crates/vpsforge-cli
```

Or:

```bash
cargo build --release
# target/release/vpsforge
```

The binary is intended to run on Linux. Detection, planning, blueprint export, and the TUI also work on the machine you use to develop VPSForge.

## Quick start

```bash
vpsforge                  # interactive TUI
vpsforge detect           # inspect the machine
vpsforge install ai       # plan + confirm
vpsforge install ai storage devops
vpsforge install storage --postgres --redis --minio
vpsforge doctor
vpsforge security audit
```

Non-interactive:

```bash
sudo vpsforge install production --yes
sudo vpsforge install ai --dry-run
```

## Profiles

| Profile        | Description                          |
|----------------|--------------------------------------|
| `ai`           | AI/ML development server             |
| `storage`      | Databases and storage services       |
| `server`       | Web/application hosting              |
| `devops`       | Container/automation environment     |
| `security`     | Defensive security/monitoring        |
| `development`  | General software development         |
| `production`   | Hardened production baseline         |
| `minimal`      | Minimal useful VPS                   |

```bash
vpsforge profile list
vpsforge profile show ai
```

Conditional example from the AI profile:

```
AI
 ├── Core
 ├── Python
 ├── Container Runtime
 └── NVIDIA          ← only if an NVIDIA GPU exists
```

## Blueprints

```bash
vpsforge blueprint create ai-production --profiles production --profiles ai --profiles devops
vpsforge blueprint apply ai-production
vpsforge blueprint export ai-production --format cloud-init
vpsforge blueprint export ai-production --format shell
vpsforge image build ai-production
```

A blueprint is a reproducible description of a machine — not a new Linux distribution. Packer and cloud-init are how you turn that description into a reusable image later.

```
VPSForge Blueprint
       │
       ├── cloud-init
       ├── shell bootstrap
       ├── Packer HCL
       └── documentation
               │
               ▼
     Reusable server image
```

## Security design

VPSForge often runs as root. It therefore:

- never silently pipes remote scripts into a shell
- requires HTTPS for third-party sources
- records signing keys and fingerprints when the vendor publishes them
- refuses unverified third-party payloads unless you pass `--allow-unverified`
- keeps the default **security** profile defensive (firewall, SSH hardening, fail2ban, auditd, Lynis, updates)

```bash
sudo vpsforge security audit
```

## Commands

```
vpsforge detect | doctor | status
vpsforge install <profiles...>
vpsforge remove <component>
vpsforge update | upgrade
vpsforge package search|install|remove
vpsforge profile list|show|apply
vpsforge blueprint create|edit|validate|apply|export
vpsforge checkpoint | rollback <id>
vpsforge security audit
vpsforge network scan
vpsforge service list
vpsforge image build
vpsforge ai doctor
vpsforge server create <name>
vpsforge logs | history
```

## Workspace

```
vpsforge/
├── crates/
│   ├── vpsforge-cli/
│   ├── vpsforge-core/
│   ├── vpsforge-detect/
│   ├── vpsforge-packages/
│   ├── vpsforge-profiles/
│   ├── vpsforge-installer/
│   ├── vpsforge-security/
│   ├── vpsforge-blueprint/
│   ├── vpsforge-checkpoint/
│   ├── vpsforge-image/
│   └── vpsforge-tui/
├── profiles/
├── docs/
├── tests/
└── packaging/
```

## License

MIT OR Apache-2.0
