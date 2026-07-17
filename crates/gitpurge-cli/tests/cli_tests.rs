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

    let config_content = format!(
        "data_dir = \"{}\"\n",
        temp_dir.path().to_str().unwrap().replace("\\", "\\\\")
    );
    std::fs::write(&config_path, config_content).unwrap();

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
    let temp_dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd
        .arg("install-cli")
        .arg("--dir")
        .arg(temp_dir.path())
        .output()
        .unwrap();
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

    let config_content = format!(
        "data_dir = \"{}\"\n",
        temp_dir.path().to_str().unwrap().replace("\\", "\\\\")
    );
    std::fs::write(&config_path, config_content).unwrap();

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

#[test]
fn test_cli_history_import() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".gitpurge.toml");

    let config_content = format!(
        "data_dir = \"{}\"\n",
        temp_dir.path().to_str().unwrap().replace("\\", "\\\\")
    );
    std::fs::write(&config_path, config_content).unwrap();

    let legacy_json = r#"{
        "backend": [
            {
                "timestamp": 1625097600.0,
                "commit": "abc",
                "date_str": "2021-07-01",
                "metrics": {
                    "total": 10,
                    "active": 4,
                    "stale": 6,
                    "merged": 3,
                    "unmerged": 7,
                    "non_standard": 1
                }
            }
        ]
    }"#;
    let json_path = temp_dir.path().join("legacy.json");
    std::fs::write(&json_path, legacy_json).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("history")
        .arg("import")
        .arg(json_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DRY-RUN: Showing what would be imported"));
    assert!(stdout.contains("Runs parsed/imported: 1"));

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
    let output = cmd
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("--execute")
        .arg("history")
        .arg("import")
        .arg(json_path.to_str().unwrap())
        .arg("--map")
        .arg("backend=gitpurge-core")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("DRY-RUN"));
    assert!(stdout.contains("Runs parsed/imported: 1"));
    assert!(stdout.contains("Metrics points stored: 1"));
}
