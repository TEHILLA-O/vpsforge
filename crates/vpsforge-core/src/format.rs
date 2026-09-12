//! Human-readable size and table helpers used by reports.

pub fn bytes_human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn gib(bytes: u64) -> String {
    let value = bytes as f64 / 1024.0 / 1024.0 / 1024.0;
    if value >= 10.0 {
        format!("{value:.0} GB")
    } else {
        format!("{value:.1} GB")
    }
}

pub fn rule(width: usize) -> String {
    "─".repeat(width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_gib() {
        assert_eq!(gib(16 * 1024 * 1024 * 1024), "16 GB");
        assert_eq!(bytes_human(1536), "1.5 KB");
    }
}
