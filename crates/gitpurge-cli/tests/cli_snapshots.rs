//! Snapshot tests for the git-purge CLI (P9-T4).

use gitpurge_core::testkit;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

struct TestEnv {
    temp_dir: TempDir,
    config_path: PathBuf,
    repo_path: PathBuf,
}

impl TestEnv {
    fn new(repo: &testkit::FixtureRepo) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".gitpurge.toml");
        let repo_path = repo.path().to_path_buf();

        let config_content = format!(
            "data_dir = \"{}\"\nbackups_root = \"{}\"\n",
            temp_dir
                .path()
                .join("data")
                .to_str()
                .unwrap()
                .replace("\\", "\\\\"),
            temp_dir
                .path()
                .join("backups")
                .to_str()
                .unwrap()
                .replace("\\", "\\\\")
        );
        std::fs::write(&config_path, config_content).unwrap();

        Self {
            temp_dir,
            config_path,
            repo_path,
        }
    }

    fn run_cli(&self, args: &[&str]) -> (String, bool) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_git-purge"));
        cmd.arg("--config").arg(&self.config_path).arg("--no-color");

        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd.output().expect("Failed to run git-purge CLI");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let success = output.status.success();

        let sanitized_stdout = self.sanitize(&stdout);

        (sanitized_stdout, success)
    }

    fn sanitize(&self, input: &str) -> String {
        let mut out = input.to_string();

        // 1. Sanitize Temp Directory Path
        let temp_str = self.temp_dir.path().to_string_lossy().into_owned();
        let temp_str_escaped = temp_str.replace("\\", "\\\\");
        out = out.replace(&temp_str, "[TEMP_DIR]");
        if !temp_str_escaped.is_empty() {
            out = out.replace(&temp_str_escaped, "[TEMP_DIR]");
        }

        // Sanitize Repo Path
        let repo_str = self.repo_path.to_string_lossy().into_owned();
        let repo_str_escaped = repo_str.replace("\\", "\\\\");
        out = out.replace(&repo_str, "[REPO_DIR]");
        if !repo_str_escaped.is_empty() {
            out = out.replace(&repo_str_escaped, "[REPO_DIR]");
        }

        // 2. Sanitize ULIDs (26 characters Base32 starting with 01)
        let ulid_re = regex::Regex::new(r"\b01[0-9A-HJKMNP-TV-Z]{24}\b").unwrap();
        out = ulid_re.replace_all(&out, "[ULID]").into_owned();

        // 3. Sanitize Short ULIDs (12 characters in history logs/run IDs)
        let short_ulid_re = regex::Regex::new(r"\b01[0-9A-HJKMNP-TV-Z]{10}\b").unwrap();
        out = short_ulid_re.replace_all(&out, "[SHORT_ULID]").into_owned();

        // 4. Sanitize 40-character Commit Hashes
        let hash_40_re = regex::Regex::new(r"\b[0-9a-fA-F]{40}\b").unwrap();
        out = hash_40_re.replace_all(&out, "[COMMIT_SHA_40]").into_owned();

        // 5. Sanitize short commit hash in JSON
        let short_hash_re = regex::Regex::new(r#""short":"[0-9a-fA-F]{7}""#).unwrap();
        out = short_hash_re
            .replace_all(&out, r#""short":"[COMMIT_SHA_7]""#)
            .into_owned();

        // 6. Sanitize date times (various formats)
        let datetime_re = regex::Regex::new(
            r"\b\d{4}-\d{2}-\d{2}[T ]\d{1,2}:\d{2}(?::\d{2})?(?:\.\d+)?(?:Z|\s*\+\d{2}:?\d{2}(?::\d{2})?|\s*UTC)?\b",
        )
        .unwrap();
        out = datetime_re.replace_all(&out, "[DATETIME]").into_owned();

        // 7. Sanitize duration / seconds/nanos in JSON
        let secs_re = regex::Regex::new(r#""secs":\d+"#).unwrap();
        out = secs_re
            .replace_all(&out, r#""secs":[SECONDS]"#)
            .into_owned();
        let nanos_re = regex::Regex::new(r#""nanos":\d+"#).unwrap();
        out = nanos_re
            .replace_all(&out, r#""nanos":[NANOS]"#)
            .into_owned();

        // 8. Sanitize relative age strings in tables / texts
        let age_re =
            regex::Regex::new(r"\b(?:\d+y(?: \d+m)?|\d+m(?: \d+d)?|\d+d|\d+h|\d+s|just now)\b")
                .unwrap();
        out = age_re.replace_all(&out, "[AGE]").into_owned();

        // 9. Sanitize user name of the actor
        if let Ok(user) = std::env::var("USER") {
            out = out.replace(&user, "[USER]");
        }
        if let Ok(username) = std::env::var("USERNAME") {
            out = out.replace(&username, "[USER]");
        }

        out
    }
}

#[test]
fn test_cli_snapshots_repo_operations() {
    let repo = testkit::merged_repo();
    let env = TestEnv::new(&repo);

    // 1. repo list (initially empty)
    let (stdout, success) = env.run_cli(&["repo", "list"]);
    assert!(success);
    insta::assert_snapshot!("repo_list_empty", stdout);

    // 2. repo add
    let (stdout, success) = env.run_cli(&[
        "repo",
        "add",
        env.repo_path.to_str().unwrap(),
        "--id",
        "test-repo",
        "--name",
        "Test Repository",
    ]);
    assert!(success);
    insta::assert_snapshot!("repo_add_success", stdout);

    // 3. repo list (with registered repo)
    let (stdout, success) = env.run_cli(&["repo", "list"]);
    assert!(success);
    insta::assert_snapshot!("repo_list_with_repo", stdout);

    // 4. repo show
    let (stdout, success) = env.run_cli(&["repo", "show", "test-repo"]);
    assert!(success);
    insta::assert_snapshot!("repo_show", stdout);
}

#[test]
fn test_cli_snapshots_scan_and_plan() {
    let repo = testkit::merged_repo();
    let env = TestEnv::new(&repo);

    // Register repo first
    env.run_cli(&[
        "repo",
        "add",
        env.repo_path.to_str().unwrap(),
        "--id",
        "test-repo",
        "--name",
        "Test Repository",
    ]);

    // 1. scan (human format)
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "scan"]);
    assert!(success);
    insta::assert_snapshot!("scan_human", stdout);

    // 2. scan (JSON format)
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "scan", "--json"]);
    assert!(success);
    insta::assert_snapshot!("scan_json", stdout);

    // 3. plan (human format)
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "plan"]);
    assert!(success);
    insta::assert_snapshot!("plan_human", stdout);

    // 4. plan (JSON format)
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "plan", "--json"]);
    assert!(success);
    insta::assert_snapshot!("plan_json", stdout);
}

#[test]
fn test_cli_snapshots_delete_and_backup() {
    let repo = testkit::merged_repo();
    let env = TestEnv::new(&repo);

    // Register repo
    env.run_cli(&[
        "repo",
        "add",
        env.repo_path.to_str().unwrap(),
        "--id",
        "test-repo",
        "--name",
        "Test Repository",
    ]);

    // 1. delete dry-run (human format)
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "delete"]);
    assert!(success);
    insta::assert_snapshot!("delete_dryrun_human", stdout);

    // 2. delete execute
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "delete", "--execute", "--yes"]);
    assert!(success);
    insta::assert_snapshot!("delete_execute_human", stdout);

    // 3. backup list
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "backup", "list"]);
    assert!(success);
    insta::assert_snapshot!("backup_list", stdout);

    // 4. history (has deleted run)
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "history"]);
    assert!(success);
    insta::assert_snapshot!("history_list", stdout);
}

#[test]
fn test_cli_snapshots_diff_and_show() {
    let repo = testkit::merged_repo();
    let env = TestEnv::new(&repo);

    // Register repo
    env.run_cli(&[
        "repo",
        "add",
        env.repo_path.to_str().unwrap(),
        "--id",
        "test-repo",
        "--name",
        "Test Repository",
    ]);

    // 1. diff (human format)
    let (stdout, success) = env.run_cli(&[
        "--repo",
        "test-repo",
        "diff",
        "main",
        "merged-branch",
        "--stat",
    ]);
    assert!(success);
    insta::assert_snapshot!("diff_stat_human", stdout);

    // 2. diff (JSON format)
    let (stdout, success) = env.run_cli(&[
        "--repo",
        "test-repo",
        "diff",
        "main",
        "merged-branch",
        "--json",
    ]);
    assert!(success);
    insta::assert_snapshot!("diff_json", stdout);

    // 3. show tree
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "show", "main"]);
    assert!(success);
    insta::assert_snapshot!("show_tree", stdout);

    // 4. show file content
    let (stdout, success) = env.run_cli(&["--repo", "test-repo", "show", "main", "file.txt"]);
    assert!(success);
    insta::assert_snapshot!("show_file", stdout);
}

#[test]
fn test_cli_snapshots_completions() {
    let repo = testkit::merged_repo();
    let env = TestEnv::new(&repo);

    let (stdout, success) = env.run_cli(&["completions", "zsh"]);
    assert!(success);
    insta::assert_snapshot!("completions_zsh", stdout);
}
