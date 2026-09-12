use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vpsforge"))
}

#[test]
fn detect_prints_report() {
    let output = bin().arg("detect").output().expect("run detect");
    assert!(output.status.success(), "{:?}", output);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("VPSFORGE SYSTEM DETECTION"));
    assert!(text.contains("Operating System"));
    assert!(text.contains("Resources"));
}

#[test]
fn profile_list() {
    let output = bin().args(["profile", "list"]).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("ai"));
    assert!(text.contains("storage"));
    assert!(text.contains("security"));
}

#[test]
fn profile_show_ai() {
    let output = bin().args(["profile", "show", "ai"]).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Python"));
    assert!(text.contains("GPU") || text.contains("conditional"));
}

#[test]
fn install_ai_dry_run() {
    let output = bin()
        .args(["install", "ai", "--dry-run", "--yes"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("PROFILE") || text.contains("PLAN") || text.contains("Python"));
}

#[test]
fn help_lists_commands() {
    let output = bin().arg("--help").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("detect"));
    assert!(text.contains("blueprint"));
    assert!(text.contains("checkpoint"));
}
