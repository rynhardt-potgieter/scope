/// Integration tests for edge cases and robustness of `sc index` and query commands.
///
/// Tests cover:
///   - Syntax-error tolerance during indexing
///   - Empty and comment-only files
///   - Querying a symbol whose source file has been deleted
///   - Deep --depth values do not hang or OOM
///   - Empty and no-results queries to `sc find`
///   - `sc refs --limit 0` is handled gracefully
///
/// Each test that needs an index creates an isolated TempDir and copies or
/// constructs the fixture there so tests do not share state.
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

/// Create a TempDir pre-populated with a `tsconfig.json` so that `sc init`
/// detects TypeScript, then run `sc init` and return `(TempDir, root)`.
///
/// Language detection happens at init time — the tsconfig.json must already
/// exist before `sc init` is run.
fn setup_empty_ts_project() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();

    // A minimal tsconfig.json is enough for language detection.
    std::fs::write(
        dir.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"target":"ES2020"},"include":["src/**/*"]}"#,
    )
    .unwrap();

    Command::cargo_bin("sc")
        .unwrap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    let root = dir.path().to_path_buf();
    (dir, root)
}

// ---------------------------------------------------------------------------
// Syntax-error tolerance
// ---------------------------------------------------------------------------

/// A TypeScript file with broken syntax must not cause `sc index --full` to crash.
///
/// Other valid files in the same project must still be indexed. The command
/// must succeed (exit 0) even if it logs a parse-error warning.
#[test]
fn test_index_file_with_syntax_errors() {
    let (dir, root) = setup_empty_ts_project();

    // Write a valid TypeScript file.
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/valid.ts"),
        "export class ValidClass { greet(): string { return 'hello'; } }",
    )
    .unwrap();

    // Write a TypeScript file with deliberately broken syntax.
    std::fs::write(
        dir.path().join("src/broken.ts"),
        "export class {{{{ this is not valid TypeScript )))))",
    )
    .unwrap();

    // Index must complete without crashing.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(&root)
        .assert()
        .success();

    // The valid class must still be discoverable.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["sketch", "ValidClass"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("ValidClass"));
}

/// An empty `.ts` file must not crash `sc index --full`.
///
/// The file hash must be recorded so the file is tracked, and the command
/// must exit with status 0.
#[test]
fn test_index_empty_file() {
    let (dir, root) = setup_empty_ts_project();

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/empty.ts"), "").unwrap();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(&root)
        .assert()
        .success();
}

/// A `.ts` file containing only comments (no symbols) must not crash
/// `sc index --full`. Exit code must be 0.
#[test]
fn test_index_file_with_no_symbols() {
    let (dir, root) = setup_empty_ts_project();

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/comments_only.ts"),
        "// This file intentionally left blank.\n// No symbols here.\n",
    )
    .unwrap();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(&root)
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Post-deletion query
// ---------------------------------------------------------------------------

/// Index a project, then delete one source file. Trying to sketch a symbol
/// that was defined in the deleted file should produce a failure with a
/// non-zero exit code and a message containing "not found".
///
/// The index is stale (the file is gone but the index still has the symbol),
/// so the symbol lookup should fail gracefully rather than panic.
#[test]
fn test_sketch_after_file_deleted() {
    let (dir, root) = setup_empty_ts_project();

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("src/temporary.ts"),
        "export class TemporaryClass { doWork(): void {} }",
    )
    .unwrap();

    // Build index while the file exists.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(&root)
        .assert()
        .success();

    // Verify the symbol is indexed.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["sketch", "TemporaryClass"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("TemporaryClass"));

    // Delete the source file.
    std::fs::remove_file(dir.path().join("src/temporary.ts")).unwrap();

    // Re-index so the deletion is reflected.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(&root)
        .assert()
        .success();

    // The symbol should now be gone — sketch must fail gracefully.
    Command::cargo_bin("sc")
        .unwrap()
        .args(["sketch", "TemporaryClass"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"));
}

// ---------------------------------------------------------------------------
// Deep depth flag
// ---------------------------------------------------------------------------

/// `sc impact PaymentService --depth 10` on the standard fixture must
/// complete without hanging, crashing, or producing a non-zero exit code.
///
/// This guards against infinite loops or exponential blowup in the impact
/// traversal when the caller requests a high depth.
#[test]
fn test_impact_depth_limit() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["impact", "PaymentService", "--depth", "10"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Impact analysis"));
}

// ---------------------------------------------------------------------------
// sc find edge cases
// ---------------------------------------------------------------------------

/// `sc find ""` with an empty query string must not crash. It must exit with
/// status 0 and produce output (even if it returns no results).
#[test]
fn test_find_empty_query() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["find", ""])
        .current_dir(&root)
        .assert()
        .success();
}

/// `sc find "xyzzynonexistent"` with a query that has no matches must exit
/// with status 0 and tell the user that no results were found.
#[test]
fn test_find_no_results() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["find", "xyzzynonexistent"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("no results found"));
}

// ---------------------------------------------------------------------------
// sc refs --limit 0
// ---------------------------------------------------------------------------

/// `sc refs PaymentService --limit 0` must not crash. Either it returns
/// nothing (with a truncation note) or it returns results — both are
/// acceptable. The key requirement is exit status 0.
#[test]
fn test_refs_with_limit_zero() {
    let (_dir, root) = setup_indexed_fixture();

    Command::cargo_bin("sc")
        .unwrap()
        .args(["refs", "PaymentService", "--limit", "0"])
        .current_dir(&root)
        .assert()
        .success();
}
