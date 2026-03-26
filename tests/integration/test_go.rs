/// Integration tests for Go language support.
///
/// Each test copies the Go fixture to a temporary directory to avoid
/// modifying the committed fixture, then drives the binary via assert_cmd.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

// Path to the committed Go fixture (relative to project root).
const GO_FIXTURE: &str = "tests/fixtures/go-simple";

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

/// Copy the Go fixture into a fresh TempDir and return it.
fn setup_go_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(GO_FIXTURE);
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

/// Index the Go fixture and open the resulting graph.db.
fn indexed_go_fixture_db() -> (rusqlite::Connection, TempDir) {
    let dir = setup_go_fixture();

    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let db_path = dir.path().join(".scope").join("graph.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    (conn, dir)
}

/// Helper to check if a symbol exists with a given name and kind.
fn symbol_exists(conn: &rusqlite::Connection, name: &str, kind: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = ?1 AND kind = ?2",
            rusqlite::params![name, kind],
            |row| row.get(0),
        )
        .unwrap();
    count > 0
}

/// Helper to get metadata JSON for a symbol by name.
fn get_metadata(conn: &rusqlite::Connection, name: &str) -> String {
    conn.query_row(
        "SELECT metadata FROM symbols WHERE name = ?1 LIMIT 1",
        rusqlite::params![name],
        |row| row.get(0),
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Tests -- scope init detects Go
// ---------------------------------------------------------------------------

#[test]
fn test_init_detects_go_from_go_mod() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("go.mod"),
        "module example.com/test\n\ngo 1.21\n",
    )
    .unwrap();

    sc_init(dir.path()).success().stdout(contains("Go"));
}

// ---------------------------------------------------------------------------
// Tests -- scope index on Go fixture
// ---------------------------------------------------------------------------

#[test]
fn test_index_full_on_go_fixture() {
    let dir = setup_go_fixture();

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

// ---------------------------------------------------------------------------
// Tests -- symbol detection (struct, interface, function, method, const)
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_go_structs() {
    let (conn, _dir) = indexed_go_fixture_db();

    assert!(
        symbol_exists(&conn, "PaymentService", "struct"),
        "PaymentService struct should be indexed"
    );
    assert!(
        symbol_exists(&conn, "PaymentResult", "struct"),
        "PaymentResult struct should be indexed"
    );
    assert!(
        symbol_exists(&conn, "CardDetails", "struct"),
        "CardDetails struct should be indexed"
    );
    assert!(
        symbol_exists(&conn, "Logger", "struct"),
        "Logger struct should be indexed"
    );
}

#[test]
fn test_index_detects_go_interfaces() {
    let (conn, _dir) = indexed_go_fixture_db();

    assert!(
        symbol_exists(&conn, "Processor", "struct"),
        "Processor interface should be indexed (kind=struct from infer_symbol_kind, refined via metadata)"
    );

    // Verify metadata marks it as an interface
    let metadata = get_metadata(&conn, "Processor");
    assert!(
        metadata.contains("\"type_kind\":\"interface\""),
        "Processor should have type_kind=interface in metadata; got: {metadata}"
    );
}

#[test]
fn test_index_detects_go_functions() {
    let (conn, _dir) = indexed_go_fixture_db();

    assert!(
        symbol_exists(&conn, "main", "function"),
        "main function should be indexed"
    );
    assert!(
        symbol_exists(&conn, "NewPaymentService", "function"),
        "NewPaymentService function should be indexed"
    );
    assert!(
        symbol_exists(&conn, "NewLogger", "function"),
        "NewLogger function should be indexed"
    );
    assert!(
        symbol_exists(&conn, "validateCard", "function"),
        "validateCard (unexported) function should be indexed"
    );
    assert!(
        symbol_exists(&conn, "calculateFee", "function"),
        "calculateFee (unexported) function should be indexed"
    );
    assert!(
        symbol_exists(&conn, "formatMessage", "function"),
        "formatMessage (unexported) function should be indexed"
    );
}

#[test]
fn test_index_detects_go_methods() {
    let (conn, _dir) = indexed_go_fixture_db();

    assert!(
        symbol_exists(&conn, "ProcessPayment", "method"),
        "ProcessPayment method should be indexed"
    );
    assert!(
        symbol_exists(&conn, "Refund", "method"),
        "Refund method should be indexed"
    );
    assert!(
        symbol_exists(&conn, "SetCurrency", "method"),
        "SetCurrency method should be indexed"
    );
    assert!(
        symbol_exists(&conn, "Info", "method"),
        "Info method should be indexed"
    );
    assert!(
        symbol_exists(&conn, "Error", "method"),
        "Error method should be indexed"
    );
}

#[test]
fn test_index_detects_go_constants() {
    let (conn, _dir) = indexed_go_fixture_db();

    assert!(
        symbol_exists(&conn, "MaxConnections", "const"),
        "MaxConnections const should be indexed"
    );
    assert!(
        symbol_exists(&conn, "DefaultCurrency", "const"),
        "DefaultCurrency const should be indexed"
    );
}

// ---------------------------------------------------------------------------
// Tests -- edge detection (imports, calls, extends)
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_go_edges() {
    let (conn, _dir) = indexed_go_fixture_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))
        .unwrap();

    assert!(
        total > 0,
        "edge count should be > 0 after indexing Go fixture; got {total}"
    );

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

    assert!(
        edge_kind_exists("imports"),
        "Go fixture should have 'imports' edges"
    );
    assert!(
        edge_kind_exists("calls"),
        "Go fixture should have 'calls' edges"
    );
    assert!(
        edge_kind_exists("extends"),
        "Go fixture should have 'extends' edges (PaymentService embeds Logger)"
    );
}

// ---------------------------------------------------------------------------
// Tests -- metadata: exported status
// ---------------------------------------------------------------------------

#[test]
fn test_index_go_exported_metadata() {
    let (conn, _dir) = indexed_go_fixture_db();

    // Exported symbol (starts with uppercase)
    let meta = get_metadata(&conn, "ProcessPayment");
    assert!(
        meta.contains("\"exported\":true"),
        "ProcessPayment should be exported; got: {meta}"
    );

    // Unexported symbol (starts with lowercase)
    let meta = get_metadata(&conn, "validateCard");
    assert!(
        meta.contains("\"exported\":false"),
        "validateCard should not be exported; got: {meta}"
    );
}

// ---------------------------------------------------------------------------
// Tests -- metadata: method receiver
// ---------------------------------------------------------------------------

#[test]
fn test_index_go_pointer_receiver_metadata() {
    let (conn, _dir) = indexed_go_fixture_db();

    // ProcessPayment has pointer receiver (*PaymentService)
    let meta = get_metadata(&conn, "ProcessPayment");
    assert!(
        meta.contains("\"receiver\":\"PaymentService\""),
        "ProcessPayment should have receiver=PaymentService; got: {meta}"
    );
    assert!(
        meta.contains("\"is_pointer_receiver\":true"),
        "ProcessPayment should have is_pointer_receiver=true; got: {meta}"
    );
}

#[test]
fn test_index_go_value_receiver_metadata() {
    let (conn, _dir) = indexed_go_fixture_db();

    // SetCurrency has value receiver (PaymentService, not *PaymentService)
    let meta = get_metadata(&conn, "SetCurrency");
    assert!(
        meta.contains("\"receiver\":\"PaymentService\""),
        "SetCurrency should have receiver=PaymentService; got: {meta}"
    );
    assert!(
        meta.contains("\"is_pointer_receiver\":false"),
        "SetCurrency should have is_pointer_receiver=false; got: {meta}"
    );
}

// ---------------------------------------------------------------------------
// Tests -- Go symbol count is reasonable
// ---------------------------------------------------------------------------

#[test]
fn test_index_go_symbol_count_is_reasonable() {
    let (conn, _dir) = indexed_go_fixture_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
        .unwrap();

    // We expect at least ~20 symbols from the fixture:
    // structs: PaymentService, PaymentResult, CardDetails, Logger
    // interfaces: Processor (stored as struct, refined via metadata)
    // type alias: Currency (stored as struct, refined via metadata)
    // functions: main, NewPaymentService, NewLogger, validateCard, calculateFee, formatMessage
    // methods: ProcessPayment, Refund, SetCurrency, Info, Error, Debug
    // consts: MaxConnections, DefaultCurrency
    assert!(
        total >= 18,
        "expected at least 18 symbols from Go fixture; got {total}"
    );
}

// ---------------------------------------------------------------------------
// Tests -- scope sketch on Go symbols
// ---------------------------------------------------------------------------

#[test]
fn test_sketch_go_struct() {
    let dir = setup_go_fixture();
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
fn test_sketch_go_struct_json() {
    let dir = setup_go_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");

    assert_eq!(json["command"], "sketch");
}

// ---------------------------------------------------------------------------
// Tests -- scope refs on Go symbols
// ---------------------------------------------------------------------------

#[test]
fn test_refs_finds_go_callers() {
    let dir = setup_go_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    // validateCard is called from ProcessPayment
    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "validateCard"])
        .current_dir(dir.path())
        .assert()
        .success();
}
