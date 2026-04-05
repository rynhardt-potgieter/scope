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

    // --json must emit exactly one parseable JSON document on stdout.
    let json: serde_json::Value = serde_json::from_slice(&output)
        .expect("setup --json should produce a single valid JSON document");
    assert_eq!(json["command"], "setup");
    assert!(json["data"]["indexed"].as_bool().unwrap_or(false));
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

    // Read CLAUDE.md after first run
    let claude_md_first = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap_or_default();
    let count_first = claude_md_first.matches("## Code Navigation").count();

    // Second run should skip init and not duplicate CLAUDE.md section
    Command::cargo_bin("scope")
        .unwrap()
        .args(["setup"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("already initialised"));

    // Verify CLAUDE.md wasn't duplicated
    let claude_md_second = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap_or_default();
    let count_second = claude_md_second.matches("## Code Navigation").count();
    assert_eq!(
        count_first, count_second,
        "CLAUDE.md should not have duplicate Code Navigation sections"
    );
    assert_eq!(
        count_second, 1,
        "Should have exactly one Code Navigation section"
    );
}
