/// Integration tests for `scope map`.
///
/// Each test copies the TypeScript fixture to a temporary directory, runs
/// `scope init` + `scope index --full`, and then drives `scope map`
/// via assert_cmd.
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

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

/// scope map should show project name, file count, and symbol count.
#[test]
fn test_map_shows_project_stats() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .arg("map")
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    // Should contain file count and symbol count in the header line.
    assert!(
        stdout.contains("files") && stdout.contains("symbols"),
        "map output should show file and symbol counts in the header"
    );
}

/// scope map should show the entry points section.
#[test]
fn test_map_shows_entrypoints() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .arg("map")
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Entry points:"));
}

/// scope map should show the core symbols section.
#[test]
fn test_map_shows_core_symbols() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("scope")
        .unwrap()
        .arg("map")
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Core symbols"));
}

/// scope map --json returns valid JSON with all expected fields.
#[test]
fn test_map_json_output() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["map", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");

    assert_eq!(
        json["command"], "map",
        "JSON envelope must have command=map"
    );
    assert!(
        !json["data"].is_null(),
        "JSON envelope must have a non-null data field"
    );
    assert!(
        json["data"]["stats"]["file_count"].is_number(),
        "stats.file_count must be a number"
    );
    assert!(
        json["data"]["stats"]["symbol_count"].is_number(),
        "stats.symbol_count must be a number"
    );
    assert!(
        json["data"]["stats"]["edge_count"].is_number(),
        "stats.edge_count must be a number"
    );
    assert!(
        json["data"]["entrypoints"].is_array(),
        "entrypoints must be an array"
    );
    assert!(
        json["data"]["core_symbols"].is_array(),
        "core_symbols must be an array"
    );
    assert!(
        json["data"]["architecture"].is_array(),
        "architecture must be an array"
    );
}

/// scope map output should be compact — under 30 lines.
#[test]
fn test_map_output_is_compact() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .arg("map")
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let line_count = stdout.lines().count();
    assert!(
        line_count <= 30,
        "map output should be under 30 lines, got {line_count}"
    );
}

/// scope map --limit 3 should limit core symbols to 3.
#[test]
fn test_map_limit_core_symbols() {
    let (_dir, root) = setup_indexed_fixture();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["map", "--limit", "3"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Count lines in the core symbols section.
    let mut in_core = false;
    let mut core_lines = 0;
    for line in stdout.lines() {
        if line.starts_with("Core symbols") {
            in_core = true;
            continue;
        }
        if in_core {
            if line.is_empty() || (!line.starts_with("  ") && !line.is_empty()) {
                break;
            }
            if line.starts_with("  ") {
                core_lines += 1;
            }
        }
    }

    assert!(
        core_lines <= 3,
        "core symbols section should have at most 3 entries with --limit 3, got {core_lines}"
    );
}

/// scope map without an index should fail with a clear error.
#[test]
fn test_map_no_index_fails() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("scope")
        .unwrap()
        .arg("map")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("No .scope/"));
}
