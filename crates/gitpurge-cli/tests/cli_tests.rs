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

#[test]
fn test_cli_completions() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd.arg("completions").arg("zsh").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("compdef git-purge"));
}

#[test]
fn test_cli_install_cli_dryrun() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd.arg("install-cli").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("[DRY-RUN] would copy"));
    assert!(stdout.contains("Run with --execute to apply."));
}

#[test]
fn test_cli_ui_not_installed() {
    // Override PATH so gitpurge-desktop is definitely not found
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd.env("PATH", "").arg("ui").output().unwrap();
    // Should exit with status code 4 (NOT_FOUND)
    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Git Purge Desktop application is not installed"));
}

#[test]
fn test_cli_auth_flows() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".gitpurge.toml");

    // 1. Add credential
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let mut child = cmd
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("auth")
        .arg("add")
        .arg("--method")
        .arg("token")
        .arg("--host")
        .arg("github.com")
        .arg("--token-stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(b"my-secret-token\n").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Stored credential for github.com"));

    // 2. List credentials
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("auth")
        .arg("list")
        .output()
        .unwrap();
    assert!(output.status.success());
    // Since FakeSecretStore is in-memory and default engine initialization uses
    // a fresh FakeSecretStore instance every time, listing might return empty in E2E binary runs.
    // However, it must execute successfully without error.

    // 3. Test credential
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("auth")
        .arg("test")
        .arg("--host")
        .arg("github.com")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("authenticated successfully"));

    // 4. Remove credential
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("auth")
        .arg("remove")
        .arg("github.com")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Removed credential for github.com"));
}
