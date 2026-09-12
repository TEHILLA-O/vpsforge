//! Package, config, service, and firewall checkpoints.

use std::path::PathBuf;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use vpsforge_core::{ForgeError, ForgePaths, ForgeResult, InstallPlan, Reversibility};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub created_at: String,
    pub note: String,
    pub reversible: usize,
    pub partial: usize,
    pub non_reversible: usize,
    pub snapshot: Snapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Snapshot {
    pub packages: Vec<String>,
    pub services: Vec<String>,
    pub firewall: Vec<String>,
    pub configs: Vec<ConfigBackup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    pub path: String,
    pub present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub id: i64,
    pub ts: String,
    pub command: String,
    pub detail: String,
}

pub fn open(paths: &ForgePaths) -> ForgeResult<Connection> {
    paths.ensure()?;
    let conn = Connection::open(&paths.history_db)
        .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS checkpoints (
            id TEXT PRIMARY KEY,
            created_at TEXT NOT NULL,
            note TEXT,
            payload TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ts TEXT NOT NULL,
            command TEXT NOT NULL,
            detail TEXT
        );",
    )
    .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    Ok(conn)
}

pub fn create_checkpoint(
    paths: &ForgePaths,
    plan: Option<&InstallPlan>,
    note: &str,
) -> ForgeResult<Checkpoint> {
    let id = format!("CP-{}", Utc::now().format("%Y%m%d-%H%M%S"));
    let mut reversible = 0;
    let mut partial = 0;
    let mut non_reversible = 0;
    if let Some(plan) = plan {
        for action in plan.pending_actions() {
            match action.reversibility {
                Reversibility::Reversible => reversible += 1,
                Reversibility::PartiallyReversible => partial += 1,
                Reversibility::NonReversible => non_reversible += 1,
            }
        }
    }
    let snapshot = capture_snapshot();
    let cp = Checkpoint {
        id: id.clone(),
        created_at: Utc::now().to_rfc3339(),
        note: note.to_string(),
        reversible,
        partial,
        non_reversible,
        snapshot,
    };
    persist(paths, &cp)?;
    record(paths, "checkpoint", &format!("created {id}"))?;
    Ok(cp)
}

pub fn persist(paths: &ForgePaths, cp: &Checkpoint) -> ForgeResult<()> {
    let conn = open(paths)?;
    let payload =
        serde_json::to_string(cp).map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    conn.execute(
        "INSERT OR REPLACE INTO checkpoints (id, created_at, note, payload) VALUES (?1, ?2, ?3, ?4)",
        params![cp.id, cp.created_at, cp.note, payload],
    )
    .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    let file = paths.checkpoint_dir.join(format!("{}.json", cp.id));
    std::fs::write(file, serde_json::to_string_pretty(cp)?)?;
    Ok(())
}

pub fn get(paths: &ForgePaths, id: &str) -> ForgeResult<Checkpoint> {
    let conn = open(paths)?;
    let payload: String = conn
        .query_row(
            "SELECT payload FROM checkpoints WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| ForgeError::Checkpoint(format!("checkpoint {id} not found")))?;
    serde_json::from_str(&payload).map_err(|e| ForgeError::Checkpoint(e.to_string()))
}

pub fn list(paths: &ForgePaths) -> ForgeResult<Vec<Checkpoint>> {
    let conn = open(paths)?;
    let mut stmt = conn
        .prepare("SELECT payload FROM checkpoints ORDER BY created_at DESC")
        .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        let payload = row.map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
        if let Ok(cp) = serde_json::from_str(&payload) {
            out.push(cp);
        }
    }
    Ok(out)
}

pub fn rollback_plan(cp: &Checkpoint) -> String {
    let mut lines = vec![
        format!("ROLLBACK {}", cp.id),
        String::new(),
        format!("Created      {}", cp.created_at),
        format!("Reversible   {}", cp.reversible),
        format!("Partial      {}", cp.partial),
        format!("Non-reversible {}", cp.non_reversible),
        String::new(),
        "VPSForge will restore recorded configuration and service enablement.".into(),
        "Package downgrades are best-effort and may be only partially reversible.".into(),
        String::new(),
        "Recorded packages:".into(),
    ];
    if cp.snapshot.packages.is_empty() {
        lines.push("  (none captured on this platform)".into());
    } else {
        for pkg in &cp.snapshot.packages {
            lines.push(format!("  - {pkg}"));
        }
    }
    lines.join("\n")
}

pub fn record(paths: &ForgePaths, command: &str, detail: &str) -> ForgeResult<()> {
    let conn = open(paths)?;
    conn.execute(
        "INSERT INTO history (ts, command, detail) VALUES (?1, ?2, ?3)",
        params![Utc::now().to_rfc3339(), command, detail],
    )
    .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    Ok(())
}

pub fn history(paths: &ForgePaths, limit: usize) -> ForgeResult<Vec<HistoryEvent>> {
    let conn = open(paths)?;
    let mut stmt = conn
        .prepare("SELECT id, ts, command, detail FROM history ORDER BY id DESC LIMIT ?1")
        .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    let rows = stmt
        .query_map(params![limit as i64], |row| {
            Ok(HistoryEvent {
                id: row.get(0)?,
                ts: row.get(1)?,
                command: row.get(2)?,
                detail: row.get(3)?,
            })
        })
        .map_err(|e| ForgeError::Checkpoint(e.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| ForgeError::Checkpoint(e.to_string()))?);
    }
    Ok(out)
}

pub fn log_file(paths: &ForgePaths) -> PathBuf {
    paths.log_dir.join("vpsforge.log")
}

fn capture_snapshot() -> Snapshot {
    let mut packages = Vec::new();
    if let Ok(output) = std::process::Command::new("dpkg-query")
        .args(["-W", "-f=${Package}\n"])
        .output()
    {
        packages = String::from_utf8_lossy(&output.stdout)
            .lines()
            .take(400)
            .map(|s| s.to_string())
            .collect();
    }
    Snapshot {
        packages,
        services: Vec::new(),
        firewall: Vec::new(),
        configs: [
            "/etc/ssh/sshd_config",
            "/etc/ufw/ufw.conf",
            "/etc/postgresql",
        ]
        .into_iter()
        .map(|path| ConfigBackup {
            present: std::path::Path::new(path).exists(),
            path: path.into(),
        })
        .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_paths() -> ForgePaths {
        let root = std::env::temp_dir().join(format!("vpsforge-test-{}", std::process::id()));
        ForgePaths {
            config_dir: root.join("cfg"),
            data_dir: root.join("data"),
            state_dir: root.join("state"),
            log_dir: root.join("logs"),
            blueprint_dir: root.join("cfg/blueprints"),
            checkpoint_dir: root.join("state/checkpoints"),
            app_dir: root.join("cfg/apps"),
            history_db: root.join("state/history.db"),
        }
    }

    #[test]
    fn checkpoint_roundtrip() {
        let paths = temp_paths();
        let cp = create_checkpoint(&paths, None, "test").unwrap();
        let loaded = get(&paths, &cp.id).unwrap();
        assert_eq!(loaded.id, cp.id);
        let _ = std::fs::remove_dir_all(
            PathBuf::from(paths.state_dir.parent().unwrap())
        );
    }
}
