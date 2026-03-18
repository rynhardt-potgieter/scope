/// Integration tests for incremental indexing (`sc index` without `--full`).
///
/// Each test copies the TypeScript fixture to a temporary directory, builds a
/// full index, then mutates files (add / modify / delete) and runs `sc index`
/// to exercise the incremental code path.
///
/// The incremental indexer reports changes to stderr (the progress channel).
/// Tests assert against `stderr` for file-change messages and against
/// `sc sketch` output to confirm the graph was updated correctly.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::{Path, PathBuf};
use std::time::Instant;
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
/// `let _dir = ...` or `let (dir, root) = ...` so the destructor does not
/// run early and delete the index while the test is still running.
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

    // Build the full index so there is a baseline to compare against.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(dir.path())
        .assert()
        .success();

    let root = dir.path().to_path_buf();
    (dir, root)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Adding a new source file is detected by the incremental indexer.
///
/// After adding `src/utils/helper.ts`, `sc index` should report the file as
/// "Added" in stderr and the new `helper` symbol should be queryable.
#[test]
fn test_incremental_detects_added_file() {
    let (_dir, root) = setup_indexed_fixture();

    // Create a new TypeScript file that did not exist in the original fixture.
    let helper_dir = root.join("src").join("utils");
    std::fs::create_dir_all(&helper_dir).unwrap();
    std::fs::write(
        helper_dir.join("helper.ts"),
        "export function helper(value: string): string {\n  return value.trim();\n}\n",
    )
    .unwrap();

    // Run incremental index and verify the added file appears in stderr.
    Command::cargo_bin("sc")
        .unwrap()
        .arg("index")
        .current_dir(&root)
        .assert()
        .success()
        .stderr(contains("Added"))
        .stderr(contains("helper.ts"));

    // The new symbol must now be in the index.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["sketch", "helper"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("helper"));
}

/// Modifying an existing source file is detected by the incremental indexer.
///
/// Overwriting `src/utils/logger.ts` with new content causes the hash to
/// change. `sc index` should report the file as "Modified" in stderr.
#[test]
fn test_incremental_detects_modified_file() {
    let (_dir, root) = setup_indexed_fixture();

    // Overwrite the existing logger file with different content.
    let logger_path = root.join("src").join("utils").join("logger.ts");
    std::fs::write(
        &logger_path,
        "export class Logger {\n\
           info(message: string): void {\n\
             console.log(message);\n\
           }\n\
           \n\
           error(message: string): void {\n\
             console.error(message);\n\
           }\n\
           \n\
           warn(message: string): void {\n\
             console.warn(message);\n\
           }\n\
         }\n",
    )
    .unwrap();

    // Run incremental index and verify the modified file appears in stderr.
    Command::cargo_bin("sc")
        .unwrap()
        .arg("index")
        .current_dir(&root)
        .assert()
        .success()
        .stderr(contains("Modified"))
        .stderr(contains("logger.ts"));
}

/// Deleting a source file is detected by the incremental indexer.
///
/// Removing `src/utils/logger.ts` from the working tree causes the indexer to
/// recognise the file as deleted. `sc index` should report "Deleted" in stderr.
#[test]
fn test_incremental_detects_deleted_file() {
    let (_dir, root) = setup_indexed_fixture();

    // Delete an existing file from the fixture copy.
    let logger_path = root.join("src").join("utils").join("logger.ts");
    std::fs::remove_file(&logger_path).unwrap();

    // Run incremental index and verify the deleted file appears in stderr.
    Command::cargo_bin("sc")
        .unwrap()
        .arg("index")
        .current_dir(&root)
        .assert()
        .success()
        .stderr(contains("Deleted"))
        .stderr(contains("logger.ts"));
}

/// When no files have changed, incremental indexing reports the index as up to date.
///
/// Running `sc index` a second time immediately after a full build should
/// detect zero changes and emit "up to date" in stderr.
#[test]
fn test_incremental_no_changes() {
    let (_dir, root) = setup_indexed_fixture();

    // No mutations — run incremental index on an already-up-to-date index.
    Command::cargo_bin("sc")
        .unwrap()
        .arg("index")
        .current_dir(&root)
        .assert()
        .success()
        .stderr(contains("up to date"));
}

/// `sc index --full` can be run a second time and rebuilds the index cleanly.
///
/// Verifies that running a full rebuild on an already-indexed project exits 0
/// and emits the symbol/file count summary to stderr.
#[test]
fn test_full_index_rebuilds_everything() {
    let (_dir, root) = setup_indexed_fixture();

    // Full rebuild on an already-indexed fixture must succeed.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(&root)
        .assert()
        .success()
        // The full-index formatter prints "N files  M symbols" for each language.
        .stderr(contains("symbols"));
}

/// Incremental indexing of a single added file completes within a generous
/// time budget suitable for CI environments.
///
/// The spec target is < 1 s. This test uses 2 s to absorb CI overhead while
/// still catching serious regressions.
#[test]
fn test_incremental_performance() {
    let (_dir, root) = setup_indexed_fixture();

    // Add a single new file to the project.
    let new_file = root.join("src").join("utils").join("perf_test_helper.ts");
    std::fs::write(
        &new_file,
        "export function perfHelper(x: number): number {\n  return x * 2;\n}\n",
    )
    .unwrap();

    let start = Instant::now();

    Command::cargo_bin("sc")
        .unwrap()
        .arg("index")
        .current_dir(&root)
        .assert()
        .success();

    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs_f64() < 2.0,
        "incremental index of a single file should complete in < 2 s, took {:.2} s",
        elapsed.as_secs_f64()
    );
}
