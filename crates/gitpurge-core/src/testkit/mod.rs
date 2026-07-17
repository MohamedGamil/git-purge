//! Testkit — deterministic fixture-repo builders (docs/12 testing-strategy).
//!
//! Behind the `testkit` feature gate so it never ships in production binaries.
//! Provides named builders that create on-disk git repos in temp dirs with fixed
//! authors, dates, and commit structure — no network, no machine-specific state.

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// A programmatic helper to construct deterministic git repositories.
pub struct FixtureRepo {
    /// The temporary directory holding the repository.
    pub dir: TempDir,
}

impl Default for FixtureRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl FixtureRepo {
    /// Create a new, initialized git repository in a temporary directory.
    pub fn new() -> Self {
        let dir = TempDir::new().expect("Failed to create temp directory");
        let repo = Self { dir };

        // Initialize git repository
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.name", "Test Author"]);
        repo.git(&["config", "user.email", "test@example.com"]);
        repo.git(&["config", "commit.gpgsign", "false"]);

        repo
    }

    /// The absolute path to the repository directory.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Write a file with the given content in the repository.
    pub fn write_file(&self, path: &str, content: &str) {
        let file_path = self.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        std::fs::write(file_path, content).expect("Failed to write test file");
    }

    /// Commit all changes currently in the repository.
    pub fn commit(&self, message: &str, date: &str) {
        self.git(&["add", "."]);
        self.git_with_env(
            &["commit", "-m", message],
            &[("GIT_AUTHOR_DATE", date), ("GIT_COMMITTER_DATE", date)],
        );
    }

    /// Run a git command in the repository.
    pub fn git(&self, args: &[&str]) -> String {
        self.git_with_env(args, &[])
    }

    /// Run a git command in the repository with extra environment variables.
    pub fn git_with_env(&self, args: &[&str], envs: &[(&str, &str)]) -> String {
        let mut cmd = Command::new("git");
        cmd.current_dir(self.path());
        cmd.args(args);
        cmd.env("GIT_CONFIG_NOSYSTEM", "1");
        cmd.env("HOME", self.path());
        for &(k, v) in envs {
            cmd.env(k, v);
        }
        let output = cmd.output().expect("Failed to run git command");
        if !output.status.success() {
            panic!(
                "git command failed: git {:?}\nstdout: {}\nstderr: {}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}

/// Create a repository with merged and unmerged branches.
pub fn merged_repo() -> FixtureRepo {
    let repo = FixtureRepo::new();
    repo.write_file("file.txt", "initial content");
    repo.commit("Initial commit", "2026-07-01T12:00:00Z");

    // merged-branch: create, add commit, then merge back to main
    repo.git(&["checkout", "-b", "merged-branch"]);
    repo.write_file("file_merged.txt", "merged branch changes");
    repo.commit("Merged branch commit", "2026-07-02T12:00:00Z");

    repo.git(&["checkout", "main"]);
    // Merge without fast-forward to ensure merge-base is clear
    let merge_date = "2026-07-04T12:00:00Z";
    repo.git_with_env(
        &[
            "merge",
            "merged-branch",
            "--no-ff",
            "-m",
            "Merge merged-branch",
        ],
        &[
            ("GIT_AUTHOR_DATE", merge_date),
            ("GIT_COMMITTER_DATE", merge_date),
        ],
    );

    // unmerged-branch: create, add commit, leave it unmerged
    repo.git(&["checkout", "-b", "unmerged-branch"]);
    repo.write_file("file_unmerged.txt", "unmerged branch changes");
    repo.commit("Unmerged branch commit", "2026-07-03T12:00:00Z");

    repo.git(&["checkout", "main"]);
    repo
}

/// Create a repository with stale and active branches.
pub fn stale_repo() -> FixtureRepo {
    let repo = FixtureRepo::new();
    repo.write_file("file.txt", "initial content");
    repo.commit("Initial commit", "2024-01-01T12:00:00Z");

    // stale-branch: committed 2 years ago, unmerged
    repo.git(&["checkout", "-b", "stale-branch"]);
    repo.write_file("file_stale.txt", "stale content");
    repo.commit("Stale commit", "2024-06-01T12:00:00Z");

    // active-branch: committed recently, unmerged
    repo.git(&["checkout", "main"]);
    repo.git(&["checkout", "-b", "active-branch"]);
    repo.write_file("file_active.txt", "active content");
    repo.commit("Active commit", "2026-07-10T12:00:00Z");

    repo.git(&["checkout", "main"]);
    repo
}

/// Create a repository configured with multiple remotes.
pub fn multi_remote_repo() -> (FixtureRepo, PathBuf, PathBuf) {
    let local = FixtureRepo::new();
    local.write_file("file.txt", "initial content");
    local.commit("Initial commit", "2026-07-01T12:00:00Z");

    let origin_path = local.path().join("remote-origin.git");
    let mut cmd = Command::new("git");
    cmd.current_dir(local.path())
        .args(["init", "--bare", "remote-origin.git"]);
    cmd.output().expect("Failed to init bare origin");

    let upstream_path = local.path().join("remote-upstream.git");
    let mut cmd2 = Command::new("git");
    cmd2.current_dir(local.path())
        .args(["init", "--bare", "remote-upstream.git"]);
    cmd2.output().expect("Failed to init bare upstream");

    // Add remotes to local repo
    local.git(&["remote", "add", "origin", &origin_path.to_string_lossy()]);
    local.git(&[
        "remote",
        "add",
        "upstream",
        &upstream_path.to_string_lossy(),
    ]);

    // Push main to origin
    local.git(&["push", "-u", "origin", "main"]);

    // Create feature/origin-only and push
    local.git(&["checkout", "-b", "feature/origin-only"]);
    local.write_file("origin.txt", "origin contents");
    local.commit("Origin only commit", "2026-07-02T12:00:00Z");
    local.git(&["push", "-u", "origin", "feature/origin-only"]);

    // Create feature/upstream-only and push to upstream
    local.git(&["checkout", "main"]);
    local.git(&["checkout", "-b", "feature/upstream-only"]);
    local.write_file("upstream.txt", "upstream contents");
    local.commit("Upstream only commit", "2026-07-03T12:00:00Z");
    local.git(&["push", "-u", "upstream", "feature/upstream-only"]);

    local.git(&["checkout", "main"]);

    (local, origin_path, upstream_path)
}

/// Create a repository containing various branch naming patterns.
pub fn naming_repo() -> FixtureRepo {
    let repo = FixtureRepo::new();
    repo.write_file("file.txt", "initial content");
    repo.commit("Initial commit", "2026-07-01T12:00:00Z");

    // Standard branches
    repo.git(&["branch", "feature/login"]);
    repo.git(&["branch", "bugfix/crash"]);
    repo.git(&["branch", "hotfix/security"]);
    repo.git(&["branch", "release/v1.0.0"]);

    // Non-standard branches
    repo.git(&["branch", "wip"]);
    repo.git(&["branch", "temp-123"]);
    repo.git(&["branch", "random_branch"]);
    repo.git(&["branch", "bugfix-no-slash"]);

    repo
}

/// Create a repository with a default branch and N other branches for benchmarking.
pub fn benchmark_repo(num_branches: usize) -> FixtureRepo {
    let repo = FixtureRepo::new();
    repo.write_file("file.txt", "initial content");
    repo.commit("Initial commit", "2026-07-01T12:00:00Z");

    for i in 0..num_branches {
        repo.git(&["branch", &format!("branch-{}", i)]);
    }

    repo
}
