/// Integration tests for Java language support.
///
/// Each test copies the Java fixture to a temporary directory to avoid
/// modifying the committed fixture, then drives the binary via assert_cmd.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

// Path to the committed Java fixture (relative to project root).
const JAVA_FIXTURE: &str = "tests/fixtures/java-simple";

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

/// Copy the Java fixture into a fresh TempDir and return it.
fn setup_java_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(JAVA_FIXTURE);
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

/// Index the Java fixture and open the resulting graph.db.
fn indexed_java_fixture_db() -> (rusqlite::Connection, TempDir) {
    let dir = setup_java_fixture();

    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let db_path = dir.path().join(".scope").join("graph.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    (conn, dir)
}

// ---------------------------------------------------------------------------
// Tests — scope init detects Java
// ---------------------------------------------------------------------------

#[test]
fn test_init_detects_java_from_pom_xml() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("pom.xml"),
        "<project><modelVersion>4.0.0</modelVersion></project>",
    )
    .unwrap();

    sc_init(dir.path()).success().stdout(contains("Java"));
}

#[test]
fn test_init_detects_java_from_build_gradle() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("build.gradle"), "apply plugin: 'java'").unwrap();

    sc_init(dir.path()).success().stdout(contains("Java"));
}

#[test]
fn test_init_detects_java_from_build_gradle_kts() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("build.gradle.kts"), "plugins { java }").unwrap();

    sc_init(dir.path()).success().stdout(contains("Java"));
}

#[test]
fn test_init_detects_java_from_src_main_java_dir() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src/main/java")).unwrap();

    sc_init(dir.path()).success().stdout(contains("Java"));
}

// ---------------------------------------------------------------------------
// Tests — scope index on Java fixture
// ---------------------------------------------------------------------------

#[test]
fn test_index_full_on_java_fixture() {
    let dir = setup_java_fixture();

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
// Tests — symbol detection (queries graph.db directly)
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_java_classes() {
    let (conn, _dir) = indexed_java_fixture_db();

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
        symbol_exists("OrderController", "class"),
        "OrderController class should be indexed"
    );
    assert!(
        symbol_exists("Logger", "class"),
        "Logger class should be indexed"
    );
    assert!(
        symbol_exists("PaymentException", "class"),
        "PaymentException class should be indexed"
    );
}

#[test]
fn test_index_detects_java_interfaces() {
    let (conn, _dir) = indexed_java_fixture_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols WHERE name = 'IPaymentClient' AND kind = 'interface'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(count > 0, "IPaymentClient interface should be indexed");
}

#[test]
fn test_index_detects_java_enums() {
    let (conn, _dir) = indexed_java_fixture_db();

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
fn test_index_detects_java_methods() {
    let (conn, _dir) = indexed_java_fixture_db();

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
        symbol_exists("processPayment", "method"),
        "processPayment method should be indexed"
    );
    assert!(
        symbol_exists("refund", "method"),
        "refund method should be indexed"
    );
    assert!(
        symbol_exists("createOrder", "method"),
        "createOrder method should be indexed"
    );
    assert!(
        symbol_exists("info", "method"),
        "info method should be indexed"
    );
}

// ---------------------------------------------------------------------------
// Tests — edge detection
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_java_import_edges() {
    let (conn, _dir) = indexed_java_fixture_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'imports'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "Java fixture should have 'imports' edges; got {count}"
    );
}

#[test]
fn test_index_detects_java_call_edges() {
    let (conn, _dir) = indexed_java_fixture_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'calls'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "Java fixture should have 'calls' edges; got {count}"
    );
}

#[test]
fn test_index_detects_java_implements_edges() {
    let (conn, _dir) = indexed_java_fixture_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'implements'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "Java fixture should have 'implements' edges (PaymentService implements IPaymentClient); got {count}"
    );
}

#[test]
fn test_index_detects_java_extends_edges() {
    let (conn, _dir) = indexed_java_fixture_db();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'extends'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "Java fixture should have 'extends' edges (PaymentException extends Exception); got {count}"
    );
}

// ---------------------------------------------------------------------------
// Tests — metadata
// ---------------------------------------------------------------------------

#[test]
fn test_index_java_metadata_has_access_modifiers() {
    let (conn, _dir) = indexed_java_fixture_db();

    // PaymentService should be public
    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'PaymentService' AND kind = 'class' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"access\":\"public\""),
        "PaymentService should have public access; got: {metadata}"
    );
}

#[test]
fn test_index_java_metadata_has_annotations() {
    let (conn, _dir) = indexed_java_fixture_db();

    // refund should have @Deprecated annotation
    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'refund' AND kind = 'method' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("Deprecated"),
        "refund method should have @Deprecated annotation; got: {metadata}"
    );
}

#[test]
fn test_index_java_metadata_has_static() {
    let (conn, _dir) = indexed_java_fixture_db();

    // getTransactionCount should be static
    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'getTransactionCount' AND kind = 'method' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"is_static\":true"),
        "getTransactionCount should be marked as static; got: {metadata}"
    );
}

#[test]
fn test_index_java_metadata_has_synchronized() {
    let (conn, _dir) = indexed_java_fixture_db();

    // refund should be synchronized
    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'refund' AND kind = 'method' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"is_synchronized\":true"),
        "refund should be marked as synchronized; got: {metadata}"
    );
}

#[test]
fn test_index_java_metadata_package_access_default() {
    let (conn, _dir) = indexed_java_fixture_db();

    // getTransactionCount has no access modifier, should default to "package"
    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'getTransactionCount' AND kind = 'method' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"access\":\"package\""),
        "getTransactionCount should have package access; got: {metadata}"
    );
}

// ---------------------------------------------------------------------------
// Tests — symbol count
// ---------------------------------------------------------------------------

#[test]
fn test_index_java_symbol_count_is_reasonable() {
    let (conn, _dir) = indexed_java_fixture_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
        .unwrap();

    // We expect at least ~20 symbols from the fixture:
    // classes: PaymentService, OrderController, Logger, PaymentException
    // interfaces: IPaymentClient
    // enums: PaymentResult
    // methods: processPayment, refund, calculateFee, getTransactionCount,
    //          createOrder, cancelOrder, info, error, create, getOrderId,
    //          isCompleted, constructors...
    // fields: logger, transactionCount, paymentService, prefix, orderId, completed
    assert!(
        total >= 15,
        "expected at least 15 symbols from Java fixture; got {total}"
    );
}

// ---------------------------------------------------------------------------
// Tests — scope sketch on Java symbols
// ---------------------------------------------------------------------------

#[test]
fn test_sketch_java_class() {
    let dir = setup_java_fixture();
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
fn test_sketch_java_class_json() {
    let dir = setup_java_fixture();
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
// Tests — scope refs on Java symbols
// ---------------------------------------------------------------------------

#[test]
fn test_refs_finds_java_callers() {
    let dir = setup_java_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    // processPayment is called from OrderController.createOrder
    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "processPayment"])
        .current_dir(dir.path())
        .assert()
        .success();
}
