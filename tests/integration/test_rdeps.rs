/// Integration tests for `scope rdeps`.
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

fn setup_indexed_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    copy_dir_all(Path::new(TS_FIXTURE), dir.path());
    Command::cargo_bin("scope").unwrap().arg("init").current_dir(dir.path()).assert().success();
    Command::cargo_bin("scope").unwrap().args(["index", "--full"]).current_dir(dir.path()).assert().success();
    let root = dir.path().to_path_buf();
    (dir, root)
}

#[test]
fn test_rdeps_shows_reverse_dependencies() {
    let (_dir, root) = setup_indexed_fixture();
    Command::cargo_bin("scope").unwrap()
        .args(["rdeps", "Logger"])
        .current_dir(&root)
        .assert()
        .success()
        .stdout(contains("Logger"));
}

#[test]
fn test_rdeps_json_output() {
    let (_dir, root) = setup_indexed_fixture();
    let output = Command::cargo_bin("scope").unwrap()
        .args(["rdeps", "Logger", "--json"])
        .current_dir(&root)
        .assert()
        .success()
        .get_output().stdout.clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "rdeps");
}

#[test]
fn test_rdeps_unknown_symbol_fails() {
    let (_dir, root) = setup_indexed_fixture();
    Command::cargo_bin("scope").unwrap()
        .args(["rdeps", "NoSuchSymbol"])
        .current_dir(&root)
        .assert()
        .failure()
        .stderr(contains("not found"));
}
