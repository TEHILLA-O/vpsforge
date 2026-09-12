# Security notes

VPSForge is a root-capable provisioner. Treat the binary like `apt` or `dnf`.

## What it will not do

- Silent `curl | bash`
- Offensive exploit helpers
- Automatic overwrite of `postgresql.conf` or `sshd_config` without an explicit flag
- Trust HTTP third-party repositories

## Third-party install path

```
Download
  → verify HTTPS
  → verify checksum / signature when published
  → show source, key, fingerprint
  → install
```

Sources without integrity metadata require `--allow-unverified` after the operator has read the plan.

## Security profile

Default posture is defensive:

- host firewall
- SSH hardening (only after a working key is confirmed in the notes)
- fail2ban
- optional CrowdSec / ClamAV
- auditd, Lynis
- tcpdump / nmap / tshark for **local** inspection
- unattended security updates

`vpsforge security audit` scores the current host. `vpsforge network scan` reads local listening tables (`/proc/net/tcp`); it is not a remote attack scanner.

## Checkpoints

Large changes should be preceded by `vpsforge checkpoint` or the automatic pre-install snapshot. Rollback is best-effort.
