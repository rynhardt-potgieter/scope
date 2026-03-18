/// Integration and snapshot tests for `scope deps`.
///
/// Each test copies the TypeScript fixture to a temporary directory (to avoid
/// modifying the committed fixture), runs `scope init` + `scope index --full`, and
/// then drives `scope deps` via assert_cmd.
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
///
/// The `TempDir` must stay alive for the duration of the test — bind it with
/// `let _dir = ...` or `let (dir, root) = ...` so the destructor does not run
/// early and delete the index while the test is still running.
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
    // Normalize backslashes first so both separators are handled.
    let root_forward = root_str.replace('\\', "/");
    let output_forward = output.replace('\\', "/");
    output_forward.replace(&*root_forward, "<PROJECT_ROOT>")
}

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

/// scope deps PaymentService should succeed and list its dependencies.
///
/// The fixture's PaymentService imports Logger and types from the payments
/// module, so the output must contain at least one dependency entry and the
/// symbol name in the header.
#[test]
fn test_deps_shows_dependencies() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["deps", "PaymentService"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("PaymentService"));
}

/// scope deps PaymentService --depth 2 should succeed and show transitive deps.
///
/// The header changes to "transitive dependencies (depth 2)" when --depth is
/// greater than 1, so we assert on that label as well.
#[test]
fn test_deps_with_depth() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["deps", "PaymentService", "--depth", "2"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("PaymentService"))
        .stdout(contains("transitive"));
}

/// scope deps src/payments/service.ts should succeed showing file-level deps.
///
/// Passing a file path triggers the file-level deps path. The output header
/// uses the normalised file path, so we check for a fragment of it.
#[test]
fn test_deps_file_level() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["deps", "src/payments/service.ts"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("service.ts"));
}

/// scope deps UnknownThing must fail with a non-zero exit code and include
/// "not found" in stderr so the caller knows what went wrong.
#[test]
fn test_deps_unknown_symbol_fails() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["deps", "UnknownThing"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"));
}

/// scope deps PaymentService --json must emit valid JSON with command="deps".
#[test]
fn test_deps_json_output() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["deps", "PaymentService", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");

    assert_eq!(
        json["command"], "deps",
        "JSON envelope must have command=deps"
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

// ---------------------------------------------------------------------------
// Snapshot tests — lock the human-readable output format
// ---------------------------------------------------------------------------

/// Snapshot the full stdout of `scope deps PaymentService`.
///
/// Any change to the deps grouped format will appear as a snapshot diff.
#[test]
fn test_deps_output_format() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("scope")
        .unwrap()
        .args(["deps", "PaymentService"])
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();

    // Redact the absolute temp-dir path so snapshots are stable across machines.
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("deps_payment_service", normalized);
}
