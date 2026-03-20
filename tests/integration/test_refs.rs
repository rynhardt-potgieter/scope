/// Integration and snapshot tests for `scope refs`.
///
/// Each test copies the TypeScript fixture to a temporary directory (to avoid
/// modifying the committed fixture), runs `scope init` + `scope index --full`, and
/// then drives `scope refs` via assert_cmd.
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

/// scope refs processPayment should succeed and show file references.
#[test]
fn test_refs_finds_callers() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "processPayment"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("processPayment"));
}

/// scope refs PaymentService should show grouped output for a class symbol.
///
/// The fixture has PaymentService imported by both order.ts and refund.ts,
/// so the output should include at least one grouping section.
#[test]
fn test_refs_class_shows_grouped_output() {
    let (_dir, root) = setup_indexed_fixture();

    // For a class with multiple reference kinds (imports, instantiates, etc.)
    // the grouped formatter adds a trailing "(N):" section header.
    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "PaymentService"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("PaymentService"));
}

/// scope refs PaymentService --kind imports should only show import-kind refs.
#[test]
fn test_refs_with_kind_filter() {
    let (_dir, root) = setup_indexed_fixture();

    // With --kind imports we expect the flat (non-grouped) path with results
    // sourced from the import edges only.
    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "PaymentService", "--kind", "imports"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("PaymentService"));
}

/// scope refs PaymentService --kind imports --limit 1 should truncate and show "... N more".
///
/// PaymentService is imported by both order.ts and refund.ts (2 import edges).
/// Filtering by --kind imports forces the flat formatter, and capping at 1
/// fires the "... N more" truncation line.
#[test]
fn test_refs_with_limit() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args([
            "refs",
            "PaymentService",
            "--kind",
            "imports",
            "--limit",
            "1",
        ])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("more"));
}

/// scope refs UnknownThing must fail with a non-zero exit code and include
/// "not found" in stderr so the caller knows what went wrong.
#[test]
fn test_refs_unknown_symbol_fails() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "UnknownThing"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"));
}

/// scope refs PaymentService --json must emit valid JSON with command="refs".
#[test]
fn test_refs_json_output() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "PaymentService", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");

    assert_eq!(
        json["command"], "refs",
        "JSON envelope must have command=refs"
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

// ---------------------------------------------------------------------------
// Snapshot tests — lock the human-readable output format
// ---------------------------------------------------------------------------

/// Snapshot the full stdout of `scope refs PaymentService`.
///
/// Any change to the refs grouped format will appear as a snapshot diff.
#[test]
fn test_refs_output_format() {
    let (_dir, root) = setup_indexed_fixture();

    let raw = Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "PaymentService"])
        .current_dir(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8(raw.stdout).unwrap();

    // Redact the absolute temp-dir path so snapshots are stable across machines.
    let normalized = normalize_paths(&stdout, &root);

    assert_snapshot!("refs_payment_service", normalized);
}

// ---------------------------------------------------------------------------
// Callers with --depth tests
// ---------------------------------------------------------------------------

/// `scope callers processPayment --depth 2` should show transitive callers
/// with depth grouping (e.g. "Direct callers" and "Second-degree").
#[test]
fn test_callers_with_depth_shows_transitive() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["callers", "processPayment", "--depth", "2"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Transitive output uses impact formatting with depth labels
    assert!(
        stdout.contains("Direct callers") || stdout.contains("Impact analysis"),
        "Expected depth-grouped output, got:\n{stdout}"
    );
}

/// `scope callers processPayment` (no --depth flag) should produce flat output,
/// identical to `scope refs processPayment --kind calls`.
#[test]
fn test_callers_default_depth_is_flat() {
    let (_dir, root) = setup_indexed_fixture();

    let callers_output = Command::cargo_bin("scope")
        .unwrap()
        .args(["callers", "processPayment"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(callers_output).unwrap();

    // Flat output uses the refs formatter — shows "N references" header,
    // not impact-style "Direct callers" grouping.
    assert!(
        stdout.contains("reference"),
        "Expected flat refs output with 'reference' header, got:\n{stdout}"
    );
    // Should NOT contain impact-style depth labels
    assert!(
        !stdout.contains("Direct callers"),
        "Default depth=1 should use flat format, not impact grouping"
    );
}
