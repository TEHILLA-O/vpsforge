# Contributing

Thanks for helping with VPSForge. Prefer small crate-scoped changes that keep detection, planning, and package execution separated.

## Prerequisites

- Rust 1.80 or newer (workspace `rust-version`)
- A Linux host for live install/package tests (detection, planning, blueprint export, and the TUI also work on the machine you develop on)
- Optional: Packer only if you exercise image export end-to-end

## Setup

```bash
cargo build --release
# or install the CLI
cargo install --path crates/vpsforge-cli
```

## Run

```bash
cargo run -p vpsforge-cli -- --help
cargo run -p vpsforge-cli -- detect
cargo run -p vpsforge-cli -- profile list
cargo run -p vpsforge-cli -- install ai --dry-run
```

## Test

```bash
cargo test --workspace
```

Format:

```bash
cargo fmt
```

## Guidelines

- Keep logical package names in profiles; map them in `vpsforge-packages`, not in YAML install scripts.
- Do not introduce silent `curl | bash` install paths. Third-party sources need HTTPS and trust metadata.
- Default security posture stays defensive (firewall, SSH hardening, fail2ban, auditd, Lynis, updates).
- Keep commit messages short and human.
