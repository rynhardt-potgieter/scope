/// Integration and snapshot tests for `sc impact`.
///
/// Each test copies the TypeScript fixture to a temporary directory (to avoid
/// modifying the committed fixture), runs `sc init` + `sc index --full`, and
/// then drives `sc impact` via assert_cmd.
///
/// Note on impact data sparsity: the current edge extraction uses `__module__`
/// synthetic IDs for `from_id` on import edges, which means the recursive CTE
/// for caller traversal may not produce deep chains in this fixture. Tests
/// verify that the command runs successfully and produces valid output format
/// rather than asserting specific caller names.
///
/// Snapshot tests use `insta`. On first run they create files under
/// `tests/snapshots/`. Run `cargo insta review` to accept new snapshots.
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

/// Copy the TypeScript fixture into a fresh TempDir, run `sc init` and
/// `sc index --full`, then return `(TempDir, project_root_path)`.
///
/// The `TempDir` must stay alive for the duration of the test — bind it with
/// `let _dir = ...` or `let (dir, root) = ...` so the destructor does not run
/// early and delete the index while the test is still running.
fn setup_indexed_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(TS_FIXTURE);
    copy_dir_all(fixture, dir.path());

    // Initialise scope config.
    Command::cargo_bin("sc")
        .unwrap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Build the full index.
    Command::cargo_bin("sc")
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

/// sc impact processPayment should succeed and produce an "Impact analysis" header.
#[test]
fn test_impact_shows_analysis() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["impact", "processPayment"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Impact analysis"));
}

/// sc impact UnknownThing must fail with a non-zero exit code and include
/// "not found" in stderr so the caller knows what went wrong.
#[test]
fn test_impact_unknown_symbol_fails() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["impact", "UnknownThing"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"));
}

/// sc impact PaymentService --json must emit valid JSON with command="impact".
#[test]
fn test_impact_json_output() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("sc")
        .unwrap()
        .args(["impact", "PaymentService", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");

    assert_eq!(
        json["command"], "impact",
        "JSON envelope must have command=impact"
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

/// sc impact PaymentService --depth 2 should succeed without errors.
///
/// Verifies that the --depth flag is accepted and the command completes
/// successfully with a valid depth override.
#[test]
fn test_impact_with_depth() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["impact", "PaymentService", "--depth", "2"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Impact analysis"));
}

// ---------------------------------------------------------------------------
// Snapshot tests — lock the human-readable output format
// ---------------------------------------------------------------------------

/// Snapshot the full stdout of `sc impact PaymentService`.
///
/// Any change to the impact output format will appear as a snapshot diff.
#[test]
fn test_impact_output_format() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("sc")
        .unwrap()
        .args(["impact", "PaymentService"])
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();

    // Redact the absolute temp-dir path so snapshots are stable across machines.
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("impact_payment_service", normalized);
}
