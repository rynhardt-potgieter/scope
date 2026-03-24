/// Integration tests for `scope index --watch`.
///
/// Tests focus on CLI flag parsing and lock file management.
/// Filesystem event tests are intentionally avoided because they are
/// inherently flaky and timing-dependent.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

// Path to the committed TypeScript fixture (relative to project root).
const TS_FIXTURE: &str = "tests/fixtures/typescript-simple";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Copy an entire directory tree into `dest`.
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

/// Copy the TypeScript fixture into a fresh TempDir, run scope init + index, and return it.
fn setup_ts_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(TS_FIXTURE);
    copy_dir_all(fixture, dir.path());

    // Run scope init to create .scope/config.toml
    Command::cargo_bin("scope")
        .unwrap()
        .args(["init"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Run scope index to create graph.db
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index"])
        .current_dir(dir.path())
        .assert()
        .success();

    dir
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn watch_flag_is_accepted_by_clap() {
    // Just verify that --watch is a valid flag and --help mentions it
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--help"])
        .assert()
        .success()
        .stdout(contains("--watch"));
}

#[test]
fn watch_flag_requires_scope_init() {
    let dir = TempDir::new().unwrap();

    // Without .scope/ dir, should fail with helpful error
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--watch"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("Run 'scope init'"));
}

#[test]
fn watch_lock_file_is_created_on_watch_start() {
    let dir = setup_ts_fixture();
    let scope_dir = dir.path().join(".scope");
    let lock_path = scope_dir.join(".watch.lock");

    // Start a watch process that we'll kill after checking the lock file
    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin("scope"))
        .args(["index", "--watch"])
        .current_dir(dir.path())
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    // Wait for lock file to appear (retry up to 10 seconds)
    let mut found = false;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if lock_path.exists() {
            found = true;
            break;
        }
    }
    assert!(found, "Lock file should be created");
    let content = std::fs::read_to_string(&lock_path).unwrap();
    let pid: u32 = content
        .trim()
        .parse()
        .expect("Lock file should contain a valid PID");
    assert!(pid > 0, "PID should be positive");

    // Kill the process
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn watch_lock_prevents_concurrent_watcher() {
    let dir = setup_ts_fixture();
    let scope_dir = dir.path().join(".scope");
    let lock_path = scope_dir.join(".watch.lock");

    // Start first watcher
    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin("scope"))
        .args(["index", "--watch"])
        .current_dir(dir.path())
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    // Wait for lock file to appear (retry up to 10 seconds)
    let mut found = false;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if lock_path.exists() {
            found = true;
            break;
        }
    }
    assert!(found, "First watcher should create lock file");

    // Try to start a second watcher — should fail
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--watch"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("Another watcher is running"));

    // Clean up
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn watch_json_flag_is_accepted() {
    // Verify --watch --json is valid (help text should mention both)
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--help"])
        .assert()
        .success()
        .stdout(contains("--watch"))
        .stdout(contains("--json"));
}

#[test]
fn watch_with_full_flag_is_accepted() {
    // Verify --watch --full is valid
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--help"])
        .assert()
        .success()
        .stdout(contains("--watch"))
        .stdout(contains("--full"));
}
