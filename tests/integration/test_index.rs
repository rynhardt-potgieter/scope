/// Integration tests for `scope init` and `scope index --full`.
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

/// Run `scope init` in `dir` and return the Command assertion handle.
fn sc_init(dir: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("scope")
        .unwrap()
        .arg("init")
        .current_dir(dir)
        .assert()
}

/// Run `scope index --full` in `dir` and return the Command assertion handle.
fn sc_index_full(dir: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(dir)
        .assert()
}

// ---------------------------------------------------------------------------
// Tests — scope init
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
// Tests — scope index
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

    // No `scope init` — index must fail.
    sc_index_full(dir.path())
        .failure()
        .code(1)
        .stderr(contains("scope init").or(contains(".scope")));
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

// ---------------------------------------------------------------------------
// Tests — TypeScript enum variant extraction
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_typescript_enum_variants() {
    let (conn, _dir) = indexed_fixture_db();

    // PaymentMethod has three members: CreditCard, BankTransfer, Wallet
    let variants: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM symbols WHERE kind = 'variant' ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    assert!(
        variants.contains(&"CreditCard".to_string()),
        "CreditCard variant should be indexed; found: {variants:?}"
    );
    assert!(
        variants.contains(&"BankTransfer".to_string()),
        "BankTransfer variant should be indexed; found: {variants:?}"
    );
    assert!(
        variants.contains(&"Wallet".to_string()),
        "Wallet variant (with initializer) should be indexed; found: {variants:?}"
    );
}

#[test]
fn test_typescript_enum_variant_has_parent_id() {
    let (conn, _dir) = indexed_fixture_db();

    // Verify at least one variant has parent_id pointing to PaymentMethod
    let parent_name: String = conn
        .query_row(
            "SELECT p.name FROM symbols v
             JOIN symbols p ON v.parent_id = p.id
             WHERE v.name = 'Wallet' AND v.kind = 'variant'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(
        parent_name, "PaymentMethod",
        "Wallet variant should have PaymentMethod as parent"
    );
}

// ---------------------------------------------------------------------------
// Tests — TypeScript edge patterns (G7)
// ---------------------------------------------------------------------------

/// TypeScript `this.method()` edge — pattern: `this.validateAmount(request.amount)`
/// in processPayment. Uses the `@this_call` query pattern in typescript/edges.scm.
///
/// The edge extractor (pattern 7) stores the bare method name as to_id.
#[test]
fn test_typescript_this_method_call_edge_detected() {
    let (conn, _dir) = indexed_fixture_db();

    // The fixture has `this.validateAmount(request.amount)` in processPayment.
    // Pattern 7 in typescript/edges.scm captures the method name and stores it as to_id.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges
             WHERE (to_id = 'validateAmount' OR to_id LIKE '%::validateAmount') AND kind = 'calls'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "this.validateAmount() call should generate a 'calls' edge with to_id='validateAmount'; got {count}"
    );
}

// Note: TypeScript enum member access (e.g. PaymentMethod.CreditCard) cannot be
// reliably detected via tree-sitter patterns without type information — the
// `member_expression` pattern is indistinguishable from regular property access.
// Enum variant refs for TypeScript are deferred to future type-aware analysis.
