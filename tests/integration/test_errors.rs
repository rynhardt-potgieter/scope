/// Integration tests for error messages produced by every sc command.
///
/// Two classes of error are tested:
///   1. No index — commands run in an empty directory with no .scope/
///   2. Unknown symbol — commands run against a valid index with a name that
///      does not exist, to verify helpful suggestions in the error output.
///
/// All error text must appear on stderr. stdout must be empty on failure.
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

/// Copy the TypeScript fixture into a fresh TempDir, run `sc init` and
/// `sc index --full`, then return `(TempDir, project_root_path)`.
fn setup_indexed_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(TS_FIXTURE);
    copy_dir_all(fixture, dir.path());

    Command::cargo_bin("sc")
        .unwrap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(dir.path())
        .assert()
        .success();

    let root = dir.path().to_path_buf();
    (dir, root)
}

/// Return a fresh TempDir with no .scope/ directory.
fn empty_dir() -> TempDir {
    TempDir::new().unwrap()
}

// ---------------------------------------------------------------------------
// No-index error tests — every command must fail with a helpful message
// ---------------------------------------------------------------------------

/// `sc sketch Foo` in a directory with no .scope/ must fail and tell the user
/// to run sc init.
#[test]
fn test_sketch_no_index() {
    let dir = empty_dir();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["sketch", "Foo"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains(".scope/").or(contains("sc init")));
}

/// `sc refs Foo` in a directory with no .scope/ must fail and tell the user
/// to run sc init.
#[test]
fn test_refs_no_index() {
    let dir = empty_dir();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["refs", "Foo"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains(".scope/").or(contains("sc init")));
}

/// `sc deps Foo` in a directory with no .scope/ must fail and tell the user
/// to run sc init.
#[test]
fn test_deps_no_index() {
    let dir = empty_dir();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["deps", "Foo"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains(".scope/").or(contains("sc init")));
}

/// `sc impact Foo` in a directory with no .scope/ must fail and tell the user
/// to run sc init.
#[test]
fn test_impact_no_index() {
    let dir = empty_dir();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["impact", "Foo"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains(".scope/").or(contains("sc init")));
}

/// `sc find "payment"` in a directory with no .scope/ must fail and tell the
/// user to run sc init.
#[test]
fn test_find_no_index() {
    let dir = empty_dir();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["find", "payment"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains(".scope/").or(contains("sc init")));
}

/// `sc status` in a directory with no .scope/ must exit with status 0 and
/// indicate that the project has not been initialised (the command is a health
/// check, not a query, so it reports "no index" rather than failing hard).
#[test]
fn test_status_no_index() {
    let dir = empty_dir();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["status"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(
            contains("not initialised")
                .or(contains("sc init"))
                .or(contains(".scope")),
        );
}

// ---------------------------------------------------------------------------
// Unknown-symbol error tests — suggest sc find in the error message
// ---------------------------------------------------------------------------

/// `sc sketch Unknown` against a valid index must fail with "not found" and
/// suggest using `sc find` for semantic search.
#[test]
fn test_sketch_unknown_symbol_suggests_find() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["sketch", "Unknown"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"))
        .stderr(contains("sc find"));
}
