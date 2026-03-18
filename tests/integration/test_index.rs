/// Integration tests for `sc init` and `sc index --full`.
///
/// Each test copies the TypeScript fixture to a temporary directory to avoid
/// modifying the committed fixture, then drives the binary via assert_cmd.
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::path::Path;
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

/// Copy the TypeScript fixture into a fresh TempDir and return it.
///
/// The TempDir must stay alive for the duration of the test — bind it with
/// `let _dir = ...` so the destructor does not run early.
fn setup_ts_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(TS_FIXTURE);
    copy_dir_all(fixture, dir.path());
    dir
}

/// Run `sc init` in `dir` and return the Command assertion handle.
fn sc_init(dir: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("sc")
        .unwrap()
        .arg("init")
        .current_dir(dir)
        .assert()
}

/// Run `sc index --full` in `dir` and return the Command assertion handle.
fn sc_index_full(dir: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("sc")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(dir)
        .assert()
}

// ---------------------------------------------------------------------------
// Tests — sc init
// ---------------------------------------------------------------------------

#[test]
fn test_init_creates_scope_directory() {
    let dir = TempDir::new().unwrap();

    // Place a tsconfig.json so language auto-detection fires.
    std::fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();

    sc_init(dir.path())
        .success()
        .stdout(contains("Initialised"));

    let scope_dir = dir.path().join(".scope");
    assert!(
        scope_dir.exists(),
        ".scope/ directory should exist after init"
    );

    let config_toml = scope_dir.join("config.toml");
    assert!(config_toml.exists(), "config.toml should exist after init");

    let config_content = std::fs::read_to_string(&config_toml).unwrap();
    assert!(
        config_content.contains("typescript"),
        "config.toml should mention typescript; got: {config_content}"
    );
}

#[test]
fn test_init_fails_if_scope_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();

    // First init succeeds.
    sc_init(dir.path()).success();

    // Second init must fail with exit code 1.
    sc_init(dir.path())
        .failure()
        .code(1)
        .stderr(contains("already"));
}

// ---------------------------------------------------------------------------
// Tests — sc index
// ---------------------------------------------------------------------------

#[test]
fn test_index_full_on_typescript_fixture() {
    let dir = setup_ts_fixture();

    sc_init(dir.path()).success();
    sc_index_full(dir.path())
        .success()
        .stderr(contains("files"))
        .stderr(contains("symbols"));

    let graph_db = dir.path().join(".scope").join("graph.db");
    assert!(graph_db.exists(), "graph.db should exist after indexing");
    assert!(
        graph_db.metadata().unwrap().len() > 0,
        "graph.db should not be empty"
    );
}

#[test]
fn test_index_requires_init_first() {
    let dir = TempDir::new().unwrap();

    // No `sc init` — index must fail.
    sc_index_full(dir.path())
        .failure()
        .code(1)
        .stderr(contains("sc init").or(contains(".scope")));
}

// ---------------------------------------------------------------------------
// Tests — symbol detection (queries graph.db directly via rusqlite)
// ---------------------------------------------------------------------------

/// Index the fixture, open the resulting graph.db, and run a query.
///
/// Returns `(rusqlite::Connection, TempDir)` — the caller must hold the
/// TempDir alive or the database file disappears.
fn indexed_fixture_db() -> (rusqlite::Connection, TempDir) {
    let dir = setup_ts_fixture();

    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let db_path = dir.path().join(".scope").join("graph.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    (conn, dir)
}

#[test]
fn test_index_detects_typescript_symbols() {
    let (conn, _dir) = indexed_fixture_db();

    // Helper: look up a symbol by name and kind.
    let symbol_exists = |name: &str, kind: &str| -> bool {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE name = ?1 AND kind = ?2",
                rusqlite::params![name, kind],
                |row| row.get(0),
            )
            .unwrap();
        count > 0
    };

    assert!(
        symbol_exists("PaymentService", "class"),
        "PaymentService class should be indexed"
    );
    assert!(
        symbol_exists("processPayment", "method"),
        "processPayment method should be indexed"
    );
    assert!(
        symbol_exists("Logger", "class"),
        "Logger class should be indexed"
    );
    assert!(
        symbol_exists("OrderController", "class"),
        "OrderController class should be indexed"
    );
}

#[test]
fn test_index_symbol_count_is_positive() {
    let (conn, _dir) = indexed_fixture_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
        .unwrap();

    assert!(
        total > 0,
        "symbol count should be > 0 after indexing fixture; got {total}"
    );
}

#[test]
fn test_index_detects_edges() {
    let (conn, _dir) = indexed_fixture_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))
        .unwrap();

    assert!(
        total > 0,
        "edge count should be > 0 after indexing fixture; got {total}"
    );

    // At least one edge of kind "imports" or "calls" should exist.
    let edge_kind_exists = |kind: &str| -> bool {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM edges WHERE kind = ?1",
                rusqlite::params![kind],
                |row| row.get(0),
            )
            .unwrap();
        count > 0
    };

    let has_calls_or_imports = edge_kind_exists("calls") || edge_kind_exists("imports");
    assert!(
        has_calls_or_imports,
        "at least one 'calls' or 'imports' edge should exist"
    );
}
