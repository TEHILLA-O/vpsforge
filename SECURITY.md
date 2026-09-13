# Security Policy

## Authorized systems only

VPSForge often runs as root on a VPS. Use it only on machines you own or administer with authorization. Do not use install or network features against hosts outside your control.

## Supported versions

Fixes land on the default branch (`main`).

## Reporting a vulnerability

Please open a private GitHub security advisory, or contact the maintainer via the GitHub profile, with:

- Affected crate (packages, installer, security, image, CLI)
- Whether trust checks, checksum/signature handling, or rollback can be bypassed
- Reproduction on a disposable VM

Do not file public issues that include weaponized install bypasses.

## Responsible use

- Prefer `--dry-run` before applying profiles on shared hosts.
- Refuse unverified third-party payloads unless you consciously pass `--allow-unverified`.
- Treat blueprint exports and Packer templates as infrastructure-as-code: review them before baking images.
- See `docs/security.md` / `docs/ARCHITECTURE.md` for design notes on HTTPS sources, signing metadata, and defensive audits.

## Scope of this policy

This document covers the VPSForge codebase and packaging. It does not authorize unauthorized access to remote systems.
