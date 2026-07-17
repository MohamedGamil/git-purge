//! Tests asserting stable exit codes (P9-T7).

use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_exit_code_success() {
    let mut cmd = Command::cargo_bin("git-purge").unwrap();
    let output = cmd.arg("completions").arg("zsh").output().unwrap();
    assert_eq!(output.status.code(), Some(0)); // EXIT_SUCCESS
}

#[test]
fn test_exit_code_usage_error() {
    let mut cmd = Command::cargo_bin("git-purge").unwrap();
    let output = cmd.arg("--invalid-flag-abc-123").output().unwrap();
    assert_eq!(output.status.code(), Some(2)); // EXIT_USAGE
}

#[test]
fn test_exit_code_config_error() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("invalid_config.toml");
    std::fs::write(&config_path, "invalid_toml = [{").unwrap();

    let mut cmd = Command::cargo_bin("git-purge").unwrap();
    let output = cmd
        .arg("--config")
        .arg(&config_path)
        .arg("repo")
        .arg("list")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3)); // EXIT_CONFIG
}

#[test]
fn test_exit_code_not_found() {
    let mut cmd = Command::cargo_bin("git-purge").unwrap();
    let output = cmd
        .arg("--repo")
        .arg("nonexistent-repo-id-456")
        .arg("scan")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4)); // EXIT_NOT_FOUND
}
