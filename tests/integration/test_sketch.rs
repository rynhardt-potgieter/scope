/// Integration and snapshot tests for `scope sketch`.
///
/// Each test copies the TypeScript fixture to a temporary directory (to avoid
/// modifying the committed fixture), runs `scope init` + `scope index --full`, and
/// then drives `scope sketch` via assert_cmd.
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

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

/// scope sketch PaymentService should show the class name, kind, and file path.
#[test]
fn test_sketch_class_shows_name_and_kind() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("PaymentService"))
        .stdout(contains("class"))
        .stdout(contains("service.ts"));
}

/// scope sketch PaymentService.processPayment should show "method" and the method name.
#[test]
fn test_sketch_method_shows_signature() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService.processPayment"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("method"))
        .stdout(contains("processPayment"));
}

/// Qualified method lookup for a second method on the same class.
#[test]
fn test_sketch_qualified_method_lookup() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService.refundPayment"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("refundPayment"));
}

/// Sketching an unknown symbol must fail with a non-zero exit code and include
/// "not found" in stderr so the caller knows what went wrong.
#[test]
fn test_sketch_unknown_symbol_fails() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "NonExistentThing"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"));
}

/// File-level sketch: passing a file path returns symbols defined in that file.
#[test]
fn test_sketch_file_level() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "src/payments/service.ts"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("PaymentService"));
}

/// scope sketch PaymentRequest should show the interface / type.
#[test]
fn test_sketch_interface() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentRequest"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("PaymentRequest"));
}

/// --json output must be valid JSON with "command" and "data" fields.
#[test]
fn test_sketch_json_output_is_valid() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");

    assert_eq!(
        json["command"], "sketch",
        "JSON envelope must have command=sketch"
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

// ---------------------------------------------------------------------------
// Snapshot tests — lock the human-readable output format
// ---------------------------------------------------------------------------

/// Snapshot the full stdout of `scope sketch PaymentService`.
///
/// Any change to the class sketch format will appear as a snapshot diff.
#[test]
fn test_sketch_class_output_format() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService"])
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();

    // Redact the absolute temp-dir path so snapshots are stable across machines.
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("sketch_class_payment_service", normalized);
}

/// Snapshot the full stdout of `scope sketch PaymentService.processPayment`.
#[test]
fn test_sketch_method_output_format() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService.processPayment"])
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("sketch_method_process_payment", normalized);
}

/// Snapshot the full stdout of `scope sketch src/payments/service.ts`.
#[test]
fn test_sketch_file_output_format() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "src/payments/service.ts"])
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("sketch_file_service_ts", normalized);
}

// ---------------------------------------------------------------------------
// Helpers for snapshot normalization
// ---------------------------------------------------------------------------

/// Replace the absolute temp-dir root with a stable placeholder so snapshots
/// do not embed machine-specific paths.
fn normalize_paths(output: &str, root: &Path) -> String {
    let root_str = root.to_string_lossy();
    // Normalize backslashes first so both separators are handled.
    let root_forward = root_str.replace('\\', "/");
    let output_forward = output.replace('\\', "/");
    output_forward.replace(&*root_forward, "<PROJECT_ROOT>")
}
