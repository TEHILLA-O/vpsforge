# Architecture

VPSForge is a Rust workspace. Each crate has one job so package-manager commands never leak into profile YAML, and profile YAML never talks to the network.

```
vpsforge-cli ──► tui / clap
      │
      ├─ detect      host facts
      ├─ profiles    YAML + conditions + graph
      ├─ packages    Apt/Dnf/Pacman/Zypper/Apk + trust
      ├─ installer   plan → preview → apply
      ├─ security    defensive audit only
      ├─ blueprint   reproducible machine spec
      ├─ checkpoint  sqlite history + snapshots
      └─ image       cloud-init / shell / Packer
```

## Detection

`vpsforge-detect` fills `HostFacts` before any plan is built:

- `/etc/os-release` for distribution family
- `sysinfo` for CPU, RAM, disks
- `nvidia-smi`, `lspci`, and sysfs for GPUs
- `systemd-detect-virt` and DMI for virtualisation
- `PATH` probes for Docker, Python, Nginx, Postgres, Ollama, …

The planner is a pure function of `HostFacts` + selected profiles + flags. That is why a 2 GB box and an L4 box produce different AI plans.

## Package abstraction

Callers install **logical** names (`nginx`, `build-essential`, `docker`).

`vpsforge-packages` maps those names onto native packages:

| Logical           | Apt                 | Dnf                      | Pacman      |
|-------------------|---------------------|--------------------------|-------------|
| nginx             | nginx               | nginx                    | nginx       |
| build-essential   | build-essential     | gcc, gcc-c++, make       | base-devel  |

Third-party sources carry homepage, HTTPS URL, signing key, and fingerprint. The executor will not pipe a downloaded script into a shell.

## Dependency graph

Profile sections declare `depends_on`. `petgraph` topologically sorts components before the plan is emitted. Duplicate logical packages across combined profiles (`ai storage devops`) collapse to one action.

## Idempotency

Each planned action records `already_satisfied` when the matching binary is on `PATH` or the unit is already active. A second `vpsforge install ai` prints ticks and exits without changing the machine.

## Checkpoints

Before apply, VPSForge writes `CP-YYYYMMDD-HHMMSS` to SQLite and JSON. Actions are tagged reversible, partially reversible, or non-reversible. Rollback explains the difference instead of pretending every `apt install` can be undone cleanly.

## Images

`vpsforge image build` does not boot QEMU for you. It writes a Packer HCL (or cloud-init / shell) document derived from the blueprint so a later pipeline can bake a golden image.

## Repository layout

```
crates/vpsforge-cli/         clap binary and command routing
crates/vpsforge-core/        shared types and errors
crates/vpsforge-detect/      HostFacts collection
crates/vpsforge-packages/    package-manager abstraction and trust metadata
crates/vpsforge-profiles/    YAML profiles, conditions, dependency graph
crates/vpsforge-installer/   plan, preview, apply
crates/vpsforge-security/    defensive audit and scoring
crates/vpsforge-blueprint/   reproducible machine specs
crates/vpsforge-checkpoint/  SQLite history and rollback records
crates/vpsforge-image/       cloud-init, shell, Packer export
crates/vpsforge-tui/         interactive UI
profiles/                    shipped profile YAML
tests/                       workspace tests
packaging/                   distribution helpers
docs/                        architecture and security notes
```

## Control flow for `vpsforge install`

1. `vpsforge-detect` fills `HostFacts` (OS, CPU, RAM, disk, GPU, virt, PATH probes).
2. Selected profiles expand into conditional sections and a dependency graph.
3. The installer emits a dry-run plan with `already_satisfied` markers.
4. After confirmation (or `--yes`), actions apply through the package abstraction.
5. A checkpoint is recorded before apply. Blueprint/image export can capture the result without claiming a new Linux distribution.
