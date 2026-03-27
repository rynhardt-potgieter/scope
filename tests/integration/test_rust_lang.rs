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

    // All methods inside `impl PaymentService` should be extracted as method symbols
    // with parent_id pointing to PaymentService.
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

// ---------------------------------------------------------------------------
// Tests — enum variant extraction
// ---------------------------------------------------------------------------

#[test]
fn test_index_detects_rust_enum_variants() {
    let (conn, _dir) = indexed_rust_db();

    // PaymentResult has two variants: Success, Failure
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
        variants.contains(&"Success".to_string()),
        "Success variant should be indexed; found: {variants:?}"
    );
    assert!(
        variants.contains(&"Failure".to_string()),
        "Failure variant should be indexed; found: {variants:?}"
    );
    assert!(
        variants.contains(&"CreditCard".to_string()),
        "CreditCard variant should be indexed; found: {variants:?}"
    );
    assert!(
        variants.contains(&"BankTransfer".to_string()),
        "BankTransfer variant should be indexed; found: {variants:?}"
    );
}

#[test]
fn test_rust_enum_variant_has_parent_id() {
    let (conn, _dir) = indexed_rust_db();

    // Verify variants have parent_id pointing to their enum
    let parent_name: String = conn
        .query_row(
            "SELECT p.name FROM symbols v
             JOIN symbols p ON v.parent_id = p.id
             WHERE v.name = 'Success' AND v.kind = 'variant'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(
        parent_name, "PaymentResult",
        "Success variant should have PaymentResult as parent"
    );
}

// ---------------------------------------------------------------------------
// Tests — impl block method-to-struct association
// ---------------------------------------------------------------------------

#[test]
fn test_rust_impl_methods_have_parent_id_to_struct() {
    let (conn, _dir) = indexed_rust_db();

    // Methods inside `impl PaymentService { ... }` should have parent_id
    // pointing to the PaymentService struct.
    let parent_name: String = conn
        .query_row(
            "SELECT p.name FROM symbols m
             JOIN symbols p ON m.parent_id = p.id
             WHERE m.name = 'process_payment' AND m.kind = 'method'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(
        parent_name, "PaymentService",
        "process_payment should have PaymentService as parent"
    );
}

#[test]
fn test_rust_impl_methods_kind_is_method() {
    let (conn, _dir) = indexed_rust_db();

    // Functions inside impl blocks should be stored with kind = 'method'
    let methods_in_impl: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT m.name FROM symbols m
                 JOIN symbols p ON m.parent_id = p.id
                 WHERE p.name = 'PaymentService' AND m.kind = 'method'
                 ORDER BY m.name",
            )
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    assert!(
        methods_in_impl.contains(&"new".to_string()),
        "new should be a method of PaymentService; found: {methods_in_impl:?}"
    );
    assert!(
        methods_in_impl.contains(&"process_payment".to_string()),
        "process_payment should be a method of PaymentService; found: {methods_in_impl:?}"
    );
    assert!(
        methods_in_impl.contains(&"refund".to_string()),
        "refund should be a method of PaymentService; found: {methods_in_impl:?}"
    );
    assert!(
        methods_in_impl.contains(&"validate_card".to_string()),
        "validate_card should be a method of PaymentService; found: {methods_in_impl:?}"
    );
}

#[test]
fn test_rust_impl_trait_for_type_targets_type() {
    let (conn, _dir) = indexed_rust_db();

    // `impl PaymentClient for MockPaymentClient` — methods should have parent_id
    // pointing to MockPaymentClient (the target type), not PaymentClient (the trait).
    let parent_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM symbols m
             JOIN symbols p ON m.parent_id = p.id
             WHERE p.name = 'MockPaymentClient' AND m.kind = 'method'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        parent_count >= 2,
        "MockPaymentClient should have at least 2 impl methods (charge, refund); got {parent_count}"
    );
}

#[test]
fn test_rust_sketch_shows_methods_for_struct() {
    let dir = setup_rust_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    // `scope sketch PaymentService` should now list methods
    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentService"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(contains("process_payment"));
}

// ---------------------------------------------------------------------------
// Tests — Rust enum variant data shapes (G9)
// ---------------------------------------------------------------------------

#[test]
fn test_rust_enum_variant_signatures_have_data_shapes() {
    let (conn, _dir) = indexed_rust_db();

    // Success { tx_id: String } — struct variant should have data shape in signature
    let sig: String = conn
        .query_row(
            "SELECT signature FROM symbols WHERE name = 'Success' AND kind = 'variant' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        sig.contains("tx_id"),
        "Success variant signature should contain field name 'tx_id'; got: {sig}"
    );
    assert!(
        sig.contains("String"),
        "Success variant signature should contain type 'String'; got: {sig}"
    );
}

#[test]
fn test_rust_enum_variant_tuple_signature() {
    let (conn, _dir) = indexed_rust_db();

    // CreditCard(CardDetails) — tuple variant should have data shape in signature
    let sig: String = conn
        .query_row(
            "SELECT signature FROM symbols WHERE name = 'CreditCard' AND kind = 'variant' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        sig.contains("CardDetails"),
        "CreditCard variant signature should contain 'CardDetails'; got: {sig}"
    );
}

#[test]
fn test_rust_enum_sketch_shows_data_shapes() {
    let dir = setup_rust_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    // `scope sketch PaymentResult` should show variant data shapes
    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentResult"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(contains("Success"))
        .stdout(contains("tx_id"))
        .stdout(contains("Failure"))
        .stdout(contains("reason"));
}

#[test]
fn test_rust_enum_sketch_json_includes_signatures() {
    let dir = setup_rust_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "PaymentResult", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "sketch --json should succeed");

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");

    let variants = json["data"]["variants"]
        .as_array()
        .expect("data.variants should be an array");

    // Check that at least one variant has a signature with data shape
    let has_sig = variants.iter().any(|v| {
        v["signature"]
            .as_str()
            .map_or(false, |s| s.contains("tx_id"))
    });

    assert!(
        has_sig,
        "At least one variant should have a signature with data shape; variants: {variants:?}"
    );
}

// ---------------------------------------------------------------------------
// Tests — Rust trait implementation edges (G9)
// ---------------------------------------------------------------------------

#[test]
fn test_rust_trait_impl_creates_implements_edge() {
    let (conn, _dir) = indexed_rust_db();

    // `impl PaymentClient for MockPaymentClient` should create an implements edge
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'implements'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "trait impl should produce 'implements' edges; got {count}"
    );
}

#[test]
fn test_rust_trait_impl_edge_points_to_trait() {
    let (conn, _dir) = indexed_rust_db();

    // The implements edge should have to_id = 'PaymentClient'
    let to_id: String = conn
        .query_row(
            "SELECT to_id FROM edges WHERE kind = 'implements' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(
        to_id, "PaymentClient",
        "implements edge should point to trait name"
    );
}

#[test]
fn test_rust_struct_sketch_shows_trait_implementations() {
    let dir = setup_rust_fixture();
    sc_init(dir.path()).success();
    sc_index_full(dir.path()).success();

    // `scope sketch MockPaymentClient` should show "implements: PaymentClient"
    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "MockPaymentClient"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(contains("implements:"))
        .stdout(contains("PaymentClient"));
}

// ---------------------------------------------------------------------------
// Tests — Rust match arm variant refs (G7)
// ---------------------------------------------------------------------------

/// Rust match arm struct pattern — `PaymentResult::Success { .. }` in check_result.
///
/// The edge extractor (patterns 10-11) stores the bare variant name as to_id
/// and uses kind='references'. We verify by matching to_id directly.
#[test]
fn test_rust_match_arm_struct_pattern_variant_ref_detected() {
    let (conn, _dir) = indexed_rust_db();

    // The fixture has `match result { PaymentResult::Success { .. } => ... }`
    // in check_result. Pattern 10 in rust/edges.scm extracts just 'Success' as to_id.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges
             WHERE to_id = 'Success' AND kind = 'references'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "match arm `PaymentResult::Success {{ .. }}` should generate a 'references' edge with to_id='Success'; got {count}"
    );
}

/// Rust match arm failure struct pattern — `PaymentResult::Failure { .. }` in check_result.
///
/// The edge extractor (pattern 10) stores the bare variant name 'Failure' as to_id.
#[test]
fn test_rust_match_arm_failure_variant_ref_detected() {
    let (conn, _dir) = indexed_rust_db();

    // The fixture has `PaymentResult::Failure { .. }` in check_result.
    // Pattern 10 in rust/edges.scm extracts just 'Failure' as to_id.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges
             WHERE to_id = 'Failure' AND kind = 'references'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "match arm `PaymentResult::Failure {{ .. }}` should generate a 'references' edge with to_id='Failure'; got {count}"
    );
}

/// Both match arm variant ref patterns produce 'references' edges in the Rust fixture.
#[test]
fn test_rust_match_arms_produce_references_edges() {
    let (conn, _dir) = indexed_rust_db();

    // Total 'references' edges should be > 0 because of match arm variant patterns.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'references'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        count > 0,
        "Rust match arm patterns should produce 'references' edges; got {count}"
    );
}
