/// Integration tests for Rust language support.
///
/// Each test copies the Rust fixture to a temporary directory, runs
/// `scope init` + `scope index --full`, and verifies symbols and edges.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

// Path to the committed Rust fixture (relative to project root).
const RUST_FIXTURE: &str = "tests/fixtures/rust-simple";

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

/// Copy the Rust fixture into a fresh TempDir and return it.
fn setup_rust_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(RUST_FIXTURE);
    copy_dir_all(fixture, dir.path());
    dir
}

/// Run `scope init` in `dir`.
fn sc_init(dir: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("scope")
        .unwrap()
        .arg("init")
        .current_dir(dir)
        .assert()
}

/// Run `scope index --full` in `dir`.
fn sc_index_full(dir: &Path) -> assert_cmd::assert::Assert {
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(dir)
        .assert()
}

/// Index the Rust fixture and open the resulting graph.db.
fn indexed_rust_db() -> (rusqlite::Connection, TempDir) {
    let dir = setup_rust_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let db_path = dir.path().join(".scope").join("graph.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    (conn, dir)
}

// ---------------------------------------------------------------------------
// Tests — scope init detects Rust
// ---------------------------------------------------------------------------

#[test]
fn test_init_detects_rust_from_cargo_toml() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\n",
    )
    .unwrap();

    sc_init(dir.path()).success().stdout(contains("Rust"));
}

// ---------------------------------------------------------------------------
// Tests — symbol extraction
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_rust_structs() {
    let (conn, _dir) = indexed_rust_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'PaymentService' AND kind = 'struct'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0, "PaymentService struct should be indexed");
}

#[test]
fn test_index_detects_rust_enums() {
    let (conn, _dir) = indexed_rust_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'PaymentResult' AND kind = 'enum'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0, "PaymentResult enum should be indexed");
}

#[test]
fn test_index_detects_rust_traits() {
    let (conn, _dir) = indexed_rust_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'PaymentClient' AND kind = 'interface'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "PaymentClient trait should be indexed as 'interface'"
    );
}

#[test]
fn test_index_detects_rust_functions_in_impl() {
    let (conn, _dir) = indexed_rust_db();

    // Methods inside impl blocks should be extracted as functions
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'process_payment'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0, "process_payment function should be indexed");
}

#[test]
fn test_index_rust_methods_inside_impl_are_indexed() {
    let (conn, _dir) = indexed_rust_db();

    // All methods inside `impl PaymentService` should be extracted as function symbols.
    // Note: Rust impl blocks don't support parent_id association (impl_item is not
    // stored as a symbol), so methods are indexed as standalone functions.
    // Parent association for Rust is deferred to a future enhancement.
    let names = ["new", "process_payment", "refund", "validate_card"];
    for name in names {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE name = ?1",
                rusqlite::params![name],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count > 0, "{name} should be indexed from impl block");
    }
}

#[test]
fn test_index_detects_rust_consts() {
    let (conn, _dir) = indexed_rust_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'MAX_PAYMENT_AMOUNT' AND kind = 'const'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0, "MAX_PAYMENT_AMOUNT const should be indexed");
}

#[test]
fn test_index_detects_rust_type_alias() {
    let (conn, _dir) = indexed_rust_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'TransactionId' AND kind = 'type'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0, "TransactionId type alias should be indexed");
}

// ---------------------------------------------------------------------------
// Tests — edge extraction
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_rust_import_edges() {
    let (conn, _dir) = indexed_rust_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'imports'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "import edges from use statements should be detected"
    );
}

#[test]
fn test_index_detects_rust_call_edges() {
    let (conn, _dir) = indexed_rust_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'calls'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0, "call edges should be detected");
}

#[test]
fn test_index_total_symbol_count() {
    let (conn, _dir) = indexed_rust_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
        .unwrap();

    // We expect: PaymentService, PaymentResult, CardDetails, PaymentMethod,
    // PaymentClient, MockPaymentClient, Logger, MAX_PAYMENT_AMOUNT, TransactionId,
    // DEFAULT_LOGGER_NAME, plus several methods (new, process_payment, refund, validate_card,
    // charge, refund, new, info, warn, error, charge, refund for MockPaymentClient)
    assert!(
        total >= 10,
        "should have at least 10 symbols from Rust fixture; got {total}"
    );
}

// ---------------------------------------------------------------------------
// Tests — sketch command
// ---------------------------------------------------------------------------

#[test]
fn test_sketch_shows_rust_struct() {
    let dir = setup_rust_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(contains("PaymentService"));
}

#[test]
fn test_sketch_json_output_for_rust() {
    let dir = setup_rust_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "sketch --json should succeed");

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");

    assert_eq!(json["command"], "sketch");
}

// ---------------------------------------------------------------------------
// Tests — metadata extraction
// ---------------------------------------------------------------------------

#[test]
fn test_rust_metadata_captures_visibility() {
    let (conn, _dir) = indexed_rust_db();

    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'process_payment' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"visibility\":\"pub\""),
        "process_payment should have pub visibility; got: {metadata}"
    );
}

#[test]
fn test_rust_metadata_captures_async() {
    let (conn, _dir) = indexed_rust_db();

    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'process_payment' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"is_async\":true"),
        "process_payment should be async; got: {metadata}"
    );
}

#[test]
fn test_rust_private_function_visibility() {
    let (conn, _dir) = indexed_rust_db();

    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'validate_card' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"visibility\":\"private\""),
        "validate_card should have private visibility; got: {metadata}"
    );
}
