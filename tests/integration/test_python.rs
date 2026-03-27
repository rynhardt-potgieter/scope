/// Integration tests for Python language support.
///
/// Each test copies the Python fixture to a temporary directory to avoid
/// modifying the committed fixture, then drives the binary via assert_cmd.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

// Path to the committed Python fixture (relative to project root).
const PY_FIXTURE: &str = "tests/fixtures/python-simple";

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

/// Copy the Python fixture into a fresh TempDir and return it.
fn setup_py_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let fixture = Path::new(PY_FIXTURE);
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

/// Index the Python fixture and open the resulting graph.db.
fn indexed_py_fixture_db() -> (rusqlite::Connection, TempDir) {
    let dir = setup_py_fixture();

    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let db_path = dir.path().join(".scope").join("graph.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    (conn, dir)
}

// ---------------------------------------------------------------------------
// Tests — scope init detects Python
// ---------------------------------------------------------------------------

#[test]
fn test_init_detects_python_from_pyproject_toml() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"test\"",
    )
    .unwrap();

    sc_init(dir.path()).success().stdout(contains("Python"));
}

#[test]
fn test_init_detects_python_from_requirements_txt() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("requirements.txt"), "flask>=2.0").unwrap();

    sc_init(dir.path()).success().stdout(contains("Python"));
}

#[test]
fn test_init_detects_python_from_setup_py() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("setup.py"), "from setuptools import setup").unwrap();

    sc_init(dir.path()).success().stdout(contains("Python"));
}

#[test]
fn test_init_detects_python_from_pipfile() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Pipfile"), "[packages]").unwrap();

    sc_init(dir.path()).success().stdout(contains("Python"));
}

// ---------------------------------------------------------------------------
// Tests — scope index on Python fixture
// ---------------------------------------------------------------------------

#[test]
fn test_index_full_on_python_fixture() {
    let dir = setup_py_fixture();

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
fn test_index_detects_python_classes() {
    let (conn, _dir) = indexed_py_fixture_db();

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
        symbol_exists("BaseLogger", "class"),
        "BaseLogger class should be indexed"
    );
    assert!(
        symbol_exists("CardDetails", "class"),
        "CardDetails class (decorated with @dataclass) should be indexed"
    );
    assert!(
        symbol_exists("PaymentResult", "class"),
        "PaymentResult class (decorated with @dataclass) should be indexed"
    );
}

#[test]
fn test_index_detects_python_functions_and_methods() {
    let (conn, _dir) = indexed_py_fixture_db();

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

    // Methods inside classes should be "function" (infer_symbol_kind maps function_definition -> "function")
    // but they get parent_id set to their enclosing class.
    assert!(
        symbol_exists("process_payment", "function"),
        "process_payment should be indexed as function"
    );
    assert!(
        symbol_exists("refund", "function"),
        "refund should be indexed as function"
    );
    assert!(
        symbol_exists("validate_card", "function"),
        "validate_card (decorated with @staticmethod) should be indexed"
    );
    assert!(
        symbol_exists("create_order", "function"),
        "create_order should be indexed"
    );
}

#[test]
fn test_index_detects_python_edges() {
    let (conn, _dir) = indexed_py_fixture_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))
        .unwrap();

    assert!(
        total > 0,
        "edge count should be > 0 after indexing Python fixture; got {total}"
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
        "Python fixture should have 'imports' edges"
    );
    assert!(
        edge_kind_exists("calls"),
        "Python fixture should have 'calls' edges"
    );
    assert!(
        edge_kind_exists("extends"),
        "Python fixture should have 'extends' edges (Logger extends BaseLogger)"
    );
}

#[test]
fn test_index_python_symbol_count_is_reasonable() {
    let (conn, _dir) = indexed_py_fixture_db();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
        .unwrap();

    // We expect at least ~15 symbols from the fixture:
    // classes: PaymentService, OrderController, Logger, BaseLogger, LogLevel,
    //          CardDetails, PaymentResult
    // functions/methods: __init__, process_payment, refund, validate_card,
    //          is_connected, _calculate_fee, __apply_discount, create_order,
    //          cancel_order, log, info, error, create
    assert!(
        total >= 15,
        "expected at least 15 symbols from Python fixture; got {total}"
    );
}

#[test]
fn test_index_python_metadata_has_access() {
    let (conn, _dir) = indexed_py_fixture_db();

    // Check that metadata contains access information
    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = '_calculate_fee' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"access\":\"private\""),
        "_calculate_fee should have private access; got: {metadata}"
    );
}

#[test]
fn test_index_python_decorated_symbols_detected() {
    let (conn, _dir) = indexed_py_fixture_db();

    // validate_card should have staticmethod decorator
    let metadata: String = conn
        .query_row(
            "SELECT metadata FROM symbols WHERE name = 'validate_card' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        metadata.contains("\"is_static\":true"),
        "validate_card should be marked as static; got: {metadata}"
    );
}

// ---------------------------------------------------------------------------
// Tests — cls.method() edge capture (G12 regression)
// ---------------------------------------------------------------------------

#[test]
fn test_python_cls_method_call_creates_edge() {
    let (conn, _dir) = indexed_py_fixture_db();

    // Logger.create() calls cls.validate_name() — this should produce a "calls" edge
    // with to_id = "cls.validate_name" from the enclosing scope of `create`.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'calls' AND to_id = 'cls.validate_name'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "cls.validate_name() call inside classmethod should produce a calls edge; got count={count}"
    );
}

// ---------------------------------------------------------------------------
// Tests — scope sketch on Python symbols
// ---------------------------------------------------------------------------

#[test]
fn test_sketch_python_class() {
    let dir = setup_py_fixture();
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
fn test_sketch_python_class_json() {
    let dir = setup_py_fixture();
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

#[test]
fn test_sketch_python_shows_decorators_on_methods() {
    let dir = setup_py_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    // validate_card has @staticmethod decorator — sketch the method directly
    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "validate_card"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("@staticmethod"),
        "Sketch should show @staticmethod decorator on validate_card. Got:\n{stdout}"
    );

    // is_connected has @property decorator
    let output2 = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "is_connected"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("@property"),
        "Sketch should show @property decorator on is_connected. Got:\n{stdout2}"
    );
}

// ---------------------------------------------------------------------------
// Tests — scope refs on Python symbols
// ---------------------------------------------------------------------------

#[test]
fn test_refs_finds_python_callers() {
    let dir = setup_py_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    // process_payment is called from OrderController.create_order
    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "process_payment"])
        .current_dir(dir.path())
        .assert()
        .success();
}
