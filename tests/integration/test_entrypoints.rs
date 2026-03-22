/// Integration tests for `scope entrypoints`.
///
/// Each test copies the TypeScript fixture to a temporary directory, runs
/// `scope init` + `scope index --full`, and then drives `scope entrypoints`
/// via assert_cmd.
///
/// Snapshot tests use `insta`. On first run they create files under
/// `tests/integration/snapshots/`. Run `cargo insta review` to accept new
/// snapshots.
use assert_cmd::Command;
use insta::assert_snapshot;
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
fn normalize_paths(output: &str, root: &Path) -> String {
    let root_str = root.to_string_lossy();
    let root_forward = root_str.replace('\\', "/");
    let output_forward = output.replace('\\', "/");
    output_forward.replace(&*root_forward, "<PROJECT_ROOT>")
}

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

/// scope entrypoints should succeed and show the header.
#[test]
fn test_entrypoints_shows_header() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .arg("entrypoints")
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Entrypoints"));
}

/// scope entrypoints should find controllers in the fixture.
#[test]
fn test_entrypoints_finds_controllers() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .arg("entrypoints")
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Controller"));
}

/// scope entrypoints --json must emit valid JSON with command="entrypoints".
#[test]
fn test_entrypoints_json_output() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["entrypoints", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");

    assert_eq!(
        json["command"], "entrypoints",
        "JSON envelope must have command=entrypoints"
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
    assert!(json["data"].is_array(), "data must be an array of groups");
}

/// scope entrypoints without an index should fail with a clear error.
#[test]
fn test_entrypoints_no_index_fails() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("scope")
        .unwrap()
        .arg("entrypoints")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("No .scope/"));
}

// ---------------------------------------------------------------------------
// Snapshot tests — lock the human-readable output format
// ---------------------------------------------------------------------------

/// Snapshot the full stdout of `scope entrypoints`.
///
/// Any change to the entrypoints human-readable format will appear as a
/// snapshot diff.
#[test]
fn test_entrypoints_human_output_snapshot() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("scope")
        .unwrap()
        .arg("entrypoints")
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();

    // Redact the absolute temp-dir path so snapshots are stable across machines.
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("entrypoints_typescript_simple", normalized);
}

/// Snapshot the full stdout of `scope entrypoints --json`.
///
/// Any change to the entrypoints JSON envelope shape will appear as a snapshot
/// diff.
#[test]
fn test_entrypoints_json_output_snapshot() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("scope")
        .unwrap()
        .args(["entrypoints", "--json"])
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("entrypoints_typescript_simple_json", normalized);
}
