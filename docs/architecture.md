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
