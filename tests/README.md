Workspace crate tests live next to the code they cover:

- `vpsforge-profiles` — YAML parse + AI graph order
- `vpsforge-installer` — constrained host skips CUDA; 16 GB Postgres tuning
- `vpsforge-packages` — logical → native mapping and Docker trust
- `vpsforge-image` — cloud-init / shell / Packer export
- `vpsforge-cli/tests` — binary smoke tests

```bash
cargo test --workspace
```
