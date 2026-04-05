/// Integration tests for `scope setup`.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const TS_FIXTURE: &str = "tests/fixtures/typescript-simple";

fn copy_dir_all(src: &Path, dest: &Path) {
    std::fs::create_dir_all(dest).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dest_path);
        } else {
            std::fs::copy(&src_path, &dest_path).unwrap();
        }
    }
}

fn fresh_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    copy_dir_all(Path::new(TS_FIXTURE), dir.path());
    let root = dir.path().to_path_buf();
    (dir, root)
}

#[test]
fn test_setup_creates_scope_dir() {
    let (_dir, root) = fresh_fixture();
    assert!(!root.join(".scope").exists());

    Command::cargo_bin("scope")
        .unwrap()
        .args(["setup"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Setup complete"));

    assert!(root.join(".scope").exists());
    assert!(root.join(".scope/graph.db").exists());
}

#[test]
fn test_setup_json_output() {
    let (_dir, root) = fresh_fixture();
    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["setup", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // JSON output may contain multiple JSON objects (init + index + setup envelope).
    // The last valid JSON line should be the setup envelope.
    let text = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = text.lines().collect();
    // Find the line containing "setup" command
    let has_setup = lines.iter().any(|l| l.contains("\"setup\""));
    assert!(has_setup, "No setup JSON in output: {text}");
}

#[test]
fn test_setup_preload_writes_claude_md() {
    let (_dir, root) = fresh_fixture();
    Command::cargo_bin("scope")
        .unwrap()
        .args(["setup", "--preload"])
        .current_dir(&root)
        .assert()
        .success();

    let claude_md = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(claude_md.contains("Code Navigation"));
    assert!(claude_md.contains("Preloaded Architecture"));
}

#[test]
fn test_setup_idempotent() {
    let (_dir, root) = fresh_fixture();

    // First run
    Command::cargo_bin("scope")
        .unwrap()
        .args(["setup"])
        .current_dir(&root)
        .assert()
        .success();

    // Second run should skip init and not duplicate CLAUDE.md section
    Command::cargo_bin("scope")
        .unwrap()
        .args(["setup"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("already initialised"));
}
