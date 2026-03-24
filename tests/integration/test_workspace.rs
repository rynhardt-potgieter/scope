/// Integration tests for `scope workspace init` and `scope workspace list`.
///
/// Each test creates temporary directory structures to simulate workspace
/// layouts with multiple Scope projects.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::Path;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal `.scope/config.toml` in the given project directory.
fn init_scope_project(project_dir: &Path, name: &str) {
    let scope_dir = project_dir.join(".scope");
    std::fs::create_dir_all(&scope_dir).unwrap();

    let config = format!(
        r#"[project]
name = "{name}"
languages = ["typescript"]

[index]
ignore = ["node_modules"]
"#
    );
    std::fs::write(scope_dir.join("config.toml"), config).unwrap();

    // Write .gitignore too (like real scope init does)
    std::fs::write(scope_dir.join(".gitignore"), "*\n").unwrap();
}

/// Create a project with a pre-built graph.db by running `scope index`.
fn init_and_index_project(project_dir: &Path, name: &str) {
    // Create project directory and a minimal source file
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("main.ts"),
        &format!("export class {name}Service {{\n  process(): void {{}}\n}}\n"),
    )
    .unwrap();

    // Write tsconfig.json so scope init detects TypeScript
    std::fs::write(
        project_dir.join("tsconfig.json"),
        r#"{"compilerOptions": {"target": "es2020"}}"#,
    )
    .unwrap();

    // Run scope init
    Command::cargo_bin("scope")
        .unwrap()
        .args(["init"])
        .current_dir(project_dir)
        .assert()
        .success();

    // Run scope index
    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(project_dir)
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// scope workspace init tests
// ---------------------------------------------------------------------------

#[test]
fn workspace_init_discovers_scope_projects() {
    let dir = TempDir::new().unwrap();

    // Create two projects with .scope/config.toml
    let api_dir = dir.path().join("api");
    let worker_dir = dir.path().join("worker");
    std::fs::create_dir_all(&api_dir).unwrap();
    std::fs::create_dir_all(&worker_dir).unwrap();
    init_scope_project(&api_dir, "api");
    init_scope_project(&worker_dir, "worker");

    Command::cargo_bin("scope")
        .unwrap()
        .args(["workspace", "init"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(contains("Found 2 projects"));

    // Verify the manifest was created
    let manifest = dir.path().join("scope-workspace.toml");
    assert!(manifest.exists(), "scope-workspace.toml should be created");

    let content = std::fs::read_to_string(&manifest).unwrap();
    assert!(content.contains("[workspace]"));
    assert!(content.contains("api"));
    assert!(content.contains("worker"));
}

#[test]
fn workspace_init_fails_if_already_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("scope-workspace.toml"),
        "[workspace]\nname = \"test\"\n\n[[workspace.members]]\npath = \"a\"\n",
    )
    .unwrap();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["workspace", "init"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("already initialized"));
}

#[test]
fn workspace_init_fails_if_no_projects_found() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["workspace", "init"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("No Scope projects found"));
}

#[test]
fn workspace_init_with_custom_name() {
    let dir = TempDir::new().unwrap();
    let api_dir = dir.path().join("api");
    std::fs::create_dir_all(&api_dir).unwrap();
    init_scope_project(&api_dir, "api");

    Command::cargo_bin("scope")
        .unwrap()
        .args(["workspace", "init", "--name", "my-platform"])
        .current_dir(dir.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join("scope-workspace.toml")).unwrap();
    assert!(content.contains("my-platform"));
}

// ---------------------------------------------------------------------------
// scope workspace list tests
// ---------------------------------------------------------------------------

#[test]
fn workspace_list_shows_members_with_status() {
    let dir = TempDir::new().unwrap();

    // Create and index one project
    let api_dir = dir.path().join("api");
    std::fs::create_dir_all(&api_dir).unwrap();
    init_and_index_project(&api_dir, "Api");

    // Create another project that is initialised but not indexed
    let worker_dir = dir.path().join("worker");
    std::fs::create_dir_all(&worker_dir).unwrap();
    init_scope_project(&worker_dir, "worker");

    // Write workspace manifest
    std::fs::write(
        dir.path().join("scope-workspace.toml"),
        r#"[workspace]
name = "test-ws"
version = 1

[[workspace.members]]
path = "api"
name = "api"

[[workspace.members]]
path = "worker"
name = "worker"
"#,
    )
    .unwrap();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["workspace", "list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(contains("test-ws"))
        .stdout(contains("api"))
        .stdout(contains("worker"));
}

#[test]
fn workspace_list_json_output() {
    let dir = TempDir::new().unwrap();

    // Create and index a project
    let api_dir = dir.path().join("api");
    std::fs::create_dir_all(&api_dir).unwrap();
    init_and_index_project(&api_dir, "Api");

    // Write workspace manifest
    std::fs::write(
        dir.path().join("scope-workspace.toml"),
        r#"[workspace]
name = "test-ws"

[[workspace.members]]
path = "api"
name = "api"
"#,
    )
    .unwrap();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["workspace", "list", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");

    assert_eq!(json["command"], "workspace list");
    assert_eq!(json["data"]["workspace_name"], "test-ws");
    assert!(json["data"]["members"].is_array());
}

#[test]
fn workspace_list_fails_without_manifest() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["workspace", "list"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("No scope-workspace.toml found"));
}

// ---------------------------------------------------------------------------
// Helpers for workspace-wide command tests
// ---------------------------------------------------------------------------

/// Set up a workspace with two indexed projects and a manifest.
fn setup_workspace_with_two_members() -> TempDir {
    let dir = TempDir::new().unwrap();

    let api_dir = dir.path().join("api");
    std::fs::create_dir_all(&api_dir).unwrap();
    init_and_index_project(&api_dir, "Api");

    let worker_dir = dir.path().join("worker");
    std::fs::create_dir_all(&worker_dir).unwrap();
    init_and_index_project(&worker_dir, "Worker");

    std::fs::write(
        dir.path().join("scope-workspace.toml"),
        r#"[workspace]
name = "test-ws"
version = 1

[[workspace.members]]
path = "api"
name = "api"

[[workspace.members]]
path = "worker"
name = "worker"
"#,
    )
    .unwrap();

    dir
}

// ---------------------------------------------------------------------------
// scope status --workspace tests
// ---------------------------------------------------------------------------

#[test]
fn status_workspace_shows_all_members() {
    let dir = setup_workspace_with_two_members();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["status", "--workspace"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(contains("test-ws"))
        .stdout(contains("api"))
        .stdout(contains("worker"))
        .stdout(contains("Total"));
}

#[test]
fn status_workspace_json_output() {
    let dir = setup_workspace_with_two_members();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["status", "--workspace", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");

    assert_eq!(json["command"], "status");
    assert_eq!(json["data"]["workspace_name"], "test-ws");
    assert!(json["data"]["members"].is_array());
    assert_eq!(json["data"]["members"].as_array().unwrap().len(), 2);
    // Totals should be present
    assert!(json["data"]["totals"]["symbol_count"].as_u64().unwrap() > 0);
}

// ---------------------------------------------------------------------------
// scope map --workspace tests
// ---------------------------------------------------------------------------

#[test]
fn map_workspace_shows_unified_stats() {
    let dir = setup_workspace_with_two_members();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["map", "--workspace"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(contains("test-ws"));
}

#[test]
fn map_workspace_json_output() {
    let dir = setup_workspace_with_two_members();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["map", "--workspace", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");

    assert_eq!(json["command"], "map");
    assert!(json["data"]["stats"]["symbol_count"].as_u64().unwrap() > 0);
}

// ---------------------------------------------------------------------------
// scope refs --workspace tests
// ---------------------------------------------------------------------------

#[test]
fn refs_workspace_searches_all_members() {
    let dir = setup_workspace_with_two_members();

    // The ApiService class exists in the api member
    Command::cargo_bin("scope")
        .unwrap()
        .args(["refs", "ApiService", "--workspace"])
        .current_dir(dir.path())
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// scope find --workspace tests
// ---------------------------------------------------------------------------

#[test]
fn find_workspace_searches_all_members() {
    let dir = setup_workspace_with_two_members();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["find", "Service", "--workspace"])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn find_workspace_json_output() {
    let dir = setup_workspace_with_two_members();

    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["find", "Service", "--workspace", "--json"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");

    assert_eq!(json["command"], "find");
}

// ---------------------------------------------------------------------------
// scope entrypoints --workspace tests
// ---------------------------------------------------------------------------

#[test]
fn entrypoints_workspace_shows_all_members() {
    let dir = setup_workspace_with_two_members();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["entrypoints", "--workspace"])
        .current_dir(dir.path())
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// --project flag tests
// ---------------------------------------------------------------------------

#[test]
fn project_flag_targets_single_member() {
    let dir = setup_workspace_with_two_members();

    // --project api should show only api's status
    Command::cargo_bin("scope")
        .unwrap()
        .args(["status", "--project", "api"])
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn project_flag_unknown_member_fails() {
    let dir = setup_workspace_with_two_members();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["status", "--project", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("not found in workspace"));
}

// ---------------------------------------------------------------------------
// --workspace on single-project-only commands gives clear error
// ---------------------------------------------------------------------------

#[test]
fn workspace_flag_on_single_project_command_errors() {
    let dir = setup_workspace_with_two_members();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["sketch", "something", "--workspace"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(contains("single project"));
}

// ---------------------------------------------------------------------------
// Nested project detection tests
// ---------------------------------------------------------------------------

#[test]
fn nested_project_is_skipped_during_indexing() {
    let dir = TempDir::new().unwrap();

    // Create the main project
    let main_dir = dir.path().join("main");
    std::fs::create_dir_all(main_dir.join("src")).unwrap();
    std::fs::write(
        main_dir.join("src").join("app.ts"),
        "export function mainApp() {}\n",
    )
    .unwrap();
    std::fs::write(
        main_dir.join("tsconfig.json"),
        r#"{"compilerOptions": {"target": "es2020"}}"#,
    )
    .unwrap();

    // Create a nested project inside the main project
    let nested_dir = main_dir.join("plugins").join("legacy");
    std::fs::create_dir_all(nested_dir.join("src")).unwrap();
    std::fs::write(
        nested_dir.join("src").join("legacy.ts"),
        "export function legacyPlugin() {}\n",
    )
    .unwrap();
    std::fs::write(
        nested_dir.join("tsconfig.json"),
        r#"{"compilerOptions": {"target": "es2020"}}"#,
    )
    .unwrap();

    // Init the nested project first
    Command::cargo_bin("scope")
        .unwrap()
        .args(["init"])
        .current_dir(&nested_dir)
        .assert()
        .success();

    // Init and index the main project
    Command::cargo_bin("scope")
        .unwrap()
        .args(["init"])
        .current_dir(&main_dir)
        .assert()
        .success();

    Command::cargo_bin("scope")
        .unwrap()
        .args(["index", "--full"])
        .current_dir(&main_dir)
        .assert()
        .success();

    // The main project should have mainApp but NOT legacyPlugin
    Command::cargo_bin("scope")
        .unwrap()
        .args(["find", "mainApp"])
        .current_dir(&main_dir)
        .assert()
        .success()
        .stdout(contains("mainApp"));

    // legacyPlugin should not be found in the main project.
    // Use --json to get structured output we can reliably parse.
    let output = Command::cargo_bin("scope")
        .unwrap()
        .args(["find", "legacyPlugin", "--json"])
        .current_dir(&main_dir)
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");
    assert_eq!(
        json["total"].as_u64().unwrap(),
        0,
        "Nested project's symbol should not be indexed by parent. Got: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
