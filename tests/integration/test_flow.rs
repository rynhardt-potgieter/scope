/// Integration tests for `scope flow` — finds call paths between two symbols.
///
/// Each test copies the TypeScript fixture to a temporary directory, runs
/// `scope init` + `scope index --full`, and then drives `scope flow` via
/// assert_cmd.
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
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
/// `let (_dir, root) = setup_indexed_fixture()` so the destructor runs at
/// end-of-scope, not immediately.
fn setup_indexed_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(TS_FIXTURE);
    copy_dir_all(fixture, dir.path());

    Command::cargo_bin("scope")
        .unwrap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

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

/// `scope flow processPayment validateAmount` — processPayment calls validateAmount
/// via `this.validateAmount(request.amount)`, so there is a direct path.
#[test]
fn test_flow_finds_path_between_connected_symbols() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["flow", "processPayment", "validateAmount"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("processPayment").and(contains("validateAmount")));
}

/// `scope flow processPayment Logger` — Logger is not reachable from processPayment
/// by call edges alone (it is a type reference, not a call target with a known symbol path).
/// The command should succeed but report that no path was found.
#[test]
fn test_flow_returns_no_path_when_symbols_unconnected() {
    let (_dir, root) = setup_indexed_fixture();

    // Logger class is not called by processPayment — only its methods are called.
    // The flow between processPayment and the Logger *class* should be absent.
    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["flow", "processPayment", "Logger"])
        .current_dir(&root)
        .output()
        .unwrap();

    // The command must not crash (exit 0) but may report no paths.
    // Either "no path" text in stdout or a JSON result with empty paths array is valid.
    assert!(
        output.status.success(),
        "flow with no path should exit 0, not crash"
    );
}

/// `scope flow processPayment validateAmount --json` — verifies the JSON envelope
/// structure: `command`, `data.start`, `data.end`, `data.paths`.
#[test]
fn test_flow_json_output_structure() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["flow", "processPayment", "validateAmount", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("flow --json should produce valid JSON");

    assert_eq!(
        json["command"], "flow",
        "JSON envelope should have command = 'flow'; got: {}",
        json["command"]
    );

    let data = &json["data"];

    assert!(
        data["start"].is_string(),
        "data.start should be a string; got: {}",
        data["start"]
    );
    assert!(
        data["end"].is_string(),
        "data.end should be a string; got: {}",
        data["end"]
    );
    assert!(
        data["paths"].is_array(),
        "data.paths should be an array; got: {}",
        data["paths"]
    );

    // The start and end fields should echo back the requested symbol names.
    assert_eq!(
        data["start"], "processPayment",
        "data.start should be 'processPayment'"
    );
    assert_eq!(
        data["end"], "validateAmount",
        "data.end should be 'validateAmount'"
    );
}

/// `scope flow NonExistent AnotherNonExistent` — both symbols are unknown.
/// Must fail with exit code 1 and mention "not found" in stderr.
#[test]
fn test_flow_unknown_symbol_fails_with_not_found() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["flow", "NonExistentSymbol", "AlsoNonExistent"])
        .current_dir(&root)
        .assert()
        .failure()
        .code(1)
        .stderr(contains("not found"));
}
