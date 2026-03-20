/// Integration tests for `scope trace`.
///
/// Each test copies the TypeScript fixture to a temporary directory, runs
/// `scope init` + `scope index --full`, and then drives `scope trace` via
/// assert_cmd.
///
/// Snapshot tests use `insta`. Run `cargo insta review` to accept new snapshots.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::{Path, PathBuf};
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

/// Copy the TypeScript fixture into a fresh TempDir, run `scope init` and
/// `scope index --full`, then return `(TempDir, project_root_path)`.
fn setup_indexed_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(TS_FIXTURE);
    copy_dir_all(fixture, dir.path());

    // Initialise scope config.
    Command::cargo_bin("scope")
        .unwrap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Build the full index.
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(dir.path())
        .assert()
        .success();

    let root = dir.path().to_path_buf();
    (dir, root)
}

/// Replace the absolute temp-dir root with a stable placeholder so snapshots
/// do not embed machine-specific paths.
#[allow(dead_code)]
fn normalize_paths(output: &str, root: &Path) -> String {
    let root_str = root.to_string_lossy();
    let root_forward = root_str.replace('\\', "/");
    let output_forward = output.replace('\\', "/");
    output_forward.replace(&*root_forward, "<PROJECT_ROOT>")
}

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

/// scope trace processPayment should succeed and show entry paths.
#[test]
fn test_trace_finds_entry_paths() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["trace", "processPayment"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("entry path"));
}

/// scope trace UnknownSymbol must fail with a non-zero exit code and
/// include "not found" in stderr.
#[test]
fn test_trace_unknown_symbol_fails() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["trace", "UnknownSymbol"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"));
}

/// scope trace processPayment --json must emit valid JSON with command="trace".
#[test]
fn test_trace_json_output() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["trace", "processPayment", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");

    assert_eq!(
        json["command"], "trace",
        "JSON envelope must have command=trace"
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
    assert!(
        json["data"]["paths"].is_array(),
        "data must contain a paths array"
    );
}

/// scope trace validateAmount should show no entry paths since it is
/// a private method with no external callers in the fixture.
#[test]
fn test_trace_no_callers() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["trace", "validateAmount"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("0 entry paths"));
}
