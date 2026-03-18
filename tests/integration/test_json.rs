/// Integration tests that validate the JSON output envelope for every scope command
/// that supports `--json`.
///
/// Every command must:
///   - Exit with status 0
///   - Emit valid JSON on stdout
///   - Have a top-level `command` field that matches the subcommand name
///   - Have a top-level `data` field that is not null
///   - Emit nothing on stderr that would contaminate the JSON stream (progress
///     output is allowed on stderr; stdout must be clean JSON)
///
/// These tests use the TypeScript fixture project.
use assert_cmd::Command;
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
/// The `TempDir` must stay alive for the duration of the test.
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

/// Parse stdout bytes as JSON and return the value. Panics with a descriptive
/// message if the bytes are not valid JSON.
fn parse_json(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is not valid JSON: {}\nraw output: {}",
            e,
            String::from_utf8_lossy(stdout)
        )
    })
}

// ---------------------------------------------------------------------------
// JSON envelope tests
// ---------------------------------------------------------------------------

/// `scope sketch PaymentService --json` must emit valid JSON with `command="sketch"`
/// and a non-null `data` field.
#[test]
fn test_sketch_json_envelope() {
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

    let json = parse_json(&output);

    assert_eq!(
        json["command"], "sketch",
        "JSON envelope must have command=sketch, got: {}",
        json["command"]
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

/// `scope refs PaymentService --json` must emit valid JSON with `command="refs"`
/// and a non-null `data` field.
#[test]
fn test_refs_json_envelope() {
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

    let json = parse_json(&output);

    assert_eq!(
        json["command"], "refs",
        "JSON envelope must have command=refs, got: {}",
        json["command"]
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

/// `scope deps PaymentService --json` must emit valid JSON with `command="deps"`
/// and a non-null `data` field.
#[test]
fn test_deps_json_envelope() {
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

    let json = parse_json(&output);

    assert_eq!(
        json["command"], "deps",
        "JSON envelope must have command=deps, got: {}",
        json["command"]
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

/// `scope impact PaymentService --json` must emit valid JSON with `command="impact"`
/// and a non-null `data` field.
#[test]
fn test_impact_json_envelope() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["impact", "PaymentService", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);

    assert_eq!(
        json["command"], "impact",
        "JSON envelope must have command=impact, got: {}",
        json["command"]
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

/// `scope find "payment" --json` must emit valid JSON with `command="find"`
/// and a non-null `data` field.
#[test]
fn test_find_json_envelope() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["find", "payment", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);

    assert_eq!(
        json["command"], "find",
        "JSON envelope must have command=find, got: {}",
        json["command"]
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
}

/// `scope status --json` must emit valid JSON with `command="status"` and a
/// non-null `data` field. This test runs against a fully-indexed fixture so
/// that `data.index_exists` is `true` and the data object is populated.
#[test]
fn test_status_json_envelope() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["status", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);

    assert_eq!(
        json["command"], "status",
        "JSON envelope must have command=status, got: {}",
        json["command"]
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
    assert_eq!(
        json["data"]["index_exists"], true,
        "data.index_exists must be true for an indexed project"
    );
}

// ---------------------------------------------------------------------------
// JSON field shape tests
// ---------------------------------------------------------------------------

/// The sketch JSON data must contain the symbol name and kind fields nested
/// under `data.symbol`.
#[test]
fn test_sketch_json_data_has_name_and_kind() {
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

    let json = parse_json(&output);
    // The sketch data envelope is: { symbol: {...}, methods: [...], ... }
    let symbol = &json["data"]["symbol"];

    assert!(
        symbol["name"].as_str().is_some(),
        "data.symbol.name must be a string, got: {}",
        symbol["name"]
    );
    assert_eq!(
        symbol["name"], "PaymentService",
        "data.symbol.name must be PaymentService"
    );
    assert!(
        symbol["kind"].as_str().is_some(),
        "data.symbol.kind must be a string, got: {}",
        symbol["kind"]
    );
}

/// The refs JSON data must be an object with a `groups` array.
///
/// The refs formatter groups results by reference kind (imports,
/// references_type, etc.), so `data.groups` is an array of group objects.
#[test]
fn test_refs_json_data_has_groups() {
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

    let json = parse_json(&output);

    assert!(
        json["data"]["groups"].is_array(),
        "refs JSON data.groups must be an array, got: {}",
        json["data"]
    );
}

/// The find JSON data must be an array. Each result must have `name`, `kind`,
/// and `score` fields.
#[test]
fn test_find_json_data_has_required_fields() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["find", "payment", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = parse_json(&output);

    assert!(json["data"].is_array(), "find JSON data must be an array");

    let results = json["data"].as_array().unwrap();
    assert!(
        !results.is_empty(),
        "find 'payment' should return at least one result"
    );

    // Every result must have name, kind, and score.
    for result in results {
        assert!(
            result["name"].as_str().is_some(),
            "each find result must have a name string, got: {result}"
        );
        assert!(
            result["kind"].as_str().is_some(),
            "each find result must have a kind string, got: {result}"
        );
        assert!(
            result["score"].as_f64().is_some(),
            "each find result must have a numeric score, got: {result}"
        );
    }
}

/// The JSON `symbol` field on sketch must match the queried symbol name.
#[test]
fn test_sketch_json_symbol_field_matches_query() {
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

    let json = parse_json(&output);

    assert_eq!(
        json["symbol"], "PaymentService",
        "JSON envelope symbol field must match the queried symbol"
    );
}
