//! End-to-end integration tests for `git-purge` CLI.

use std::process::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd.arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("git-purge"));
    assert!(stdout.contains("safely cleaning up old and stale branches"));
}

#[test]
fn test_cli_repo_list_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".gitpurge.toml");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("repo")
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No repositories tracked yet"));
}
