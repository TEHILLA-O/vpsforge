# Packaging

VPSForge is a single static-ish Rust binary (`vpsforge`).

## Release build

```bash
cargo build --release -p vpsforge-cli
```

## Linux packages

Use [nfpm](https://nfpm.goreleaser.com/) or cargo-deb against `target/release/vpsforge`.

Suggested file layout:

```
/usr/bin/vpsforge
/usr/share/vpsforge/profiles/
/etc/vpsforge/
```

## Completions

```bash
vpsforge completions bash
vpsforge completions zsh
vpsforge completions fish
vpsforge completions powershell
```

## First-boot

Prefer `vpsforge blueprint export NAME --format cloud-init` over a hand-rolled `curl | bash` user-data script.
