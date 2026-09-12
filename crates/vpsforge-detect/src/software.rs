use std::process::Command;

use vpsforge_core::{SoftwareInventory, SoftwareStatus};

pub fn detect_software() -> SoftwareInventory {
    SoftwareInventory {
        docker: probe("docker", &["--version"]),
        python: probe_python(),
        node: probe("node", &["--version"]),
        postgresql: probe_many(&[("psql", &["--version"][..]), ("postgres", &["--version"])]),
        nginx: probe("nginx", &["-v"]),
        redis: probe("redis-server", &["--version"]),
        ollama: probe("ollama", &["--version"]),
        uv: probe("uv", &["--version"]),
        git: probe("git", &["--version"]),
        rustc: probe("rustc", &["--version"]),
        go: probe("go", &["version"]),
    }
}

fn probe(bin: &str, args: &[&str]) -> SoftwareStatus {
    let path = match which::which(bin) {
        Ok(p) => p,
        Err(_) => return SoftwareStatus::missing(),
    };
    let output = Command::new(&path).args(args).output();
    let version = output.ok().and_then(|out| {
        let text = if out.stdout.is_empty() {
            String::from_utf8_lossy(&out.stderr).into_owned()
        } else {
            String::from_utf8_lossy(&out.stdout).into_owned()
        };
        first_version(&text)
    });
    SoftwareStatus {
        installed: true,
        version,
        path: Some(path.display().to_string()),
    }
}

fn probe_many(candidates: &[(&str, &[&str])]) -> SoftwareStatus {
    for (bin, args) in candidates {
        let status = probe(bin, args);
        if status.installed {
            return status;
        }
    }
    SoftwareStatus::missing()
}

fn probe_python() -> SoftwareStatus {
    for bin in ["python3", "python"] {
        let status = probe(bin, &["--version"]);
        if status.installed {
            return status;
        }
    }
    SoftwareStatus::missing()
}

fn first_version(text: &str) -> Option<String> {
    let re = regex_lite(text);
    if !re.is_empty() {
        return Some(re);
    }
    text.lines()
        .next()
        .map(|l| l.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn regex_lite(text: &str) -> String {
    let mut buf = String::new();
    let mut seen_digit = false;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            buf.push(ch);
        } else if seen_digit && (ch == '.' || ch == '-') {
            buf.push(ch);
        } else if seen_digit {
            break;
        }
    }
    buf.trim_end_matches(['.', '-']).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_version() {
        assert_eq!(first_version("Python 3.12.3").as_deref(), Some("3.12.3"));
        assert_eq!(first_version("git version 2.43.0").as_deref(), Some("2.43.0"));
    }
}
