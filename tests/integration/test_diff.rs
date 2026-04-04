/// Integration tests for `scope diff`.
use assert_cmd::Command;
use predicates::str::contains;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const TS_FIXTURE: &str = "tests/fixtures/typescript-simple";

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

fn setup_indexed_fixture_with_git() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    copy_dir_all(Path::new(TS_FIXTURE), dir.path());

    // Init git repo and commit so diff has a baseline
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output().unwrap();
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir.path())
        .output().unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(dir.path())
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output().unwrap();

    Command::cargo_bin("scope").unwrap().arg("init").current_dir(dir.path()).assert().success();
    Command::cargo_bin("scope").unwrap().args(["index", "--full"]).current_dir(dir.path()).assert().success();

    let root = dir.path().to_path_buf();
    (dir, root)
}

#[test]
fn test_diff_no_changes() {
    let (_dir, root) = setup_indexed_fixture_with_git();
    Command::cargo_bin("scope").unwrap()
        .args(["diff"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("No changes"));
}

#[test]
fn test_diff_shows_changed_symbols() {
    let (_dir, root) = setup_indexed_fixture_with_git();

    // Modify a file to create a diff
    let service_path = root.join("src/payments/service.ts");
    let content = std::fs::read_to_string(&service_path).unwrap();
    std::fs::write(&service_path, format!("{content}\n// modified\n")).unwrap();

    Command::cargo_bin("scope").unwrap()
        .args(["diff"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("service.ts"))
        .stdout(contains("PaymentService"));
}

#[test]
fn test_diff_json_output() {
    let (_dir, root) = setup_indexed_fixture_with_git();

    let service_path = root.join("src/payments/service.ts");
    let content = std::fs::read_to_string(&service_path).unwrap();
    std::fs::write(&service_path, format!("{content}\n// modified\n")).unwrap();

    let output = Command::cargo_bin("scope").unwrap()
        .args(["diff", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output().stdout.clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "diff");
    assert!(json["data"]["changed_files"].is_array());
    assert!(json["data"]["symbols"].is_array());
}
