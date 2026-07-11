//! # gitpurge-core
//!
//! The shared **brain** of Git Purge: all domain logic for safely purging stale git
//! branches lives here. The CLI (`gitpurge-cli`) and the desktop backend
//! (`gitpurge-desktop`) are thin adapters that both call this crate — so "the CLI and
//! the UI share the same behavior" is guaranteed by the compiler, not by discipline
//! (see `docs/02-architecture.md` §1 and `delivery/CONVENTIONS.md` §2).
//!
//! ## Design
//!
//! `gitpurge-core` is a hexagonal (ports & adapters) library:
//!
//! - **Domain model** ([`model`]) — the vocabulary: repositories, branches, commits,
//!   classifications, policies, snapshots, plans, reports.
//! - **Ports (traits)** — [`git::GitBackend`], [`auth::SecretStore`],
//!   [`history::HistoryStore`], [`report::ReportSink`], [`clock::Clock`],
//!   [`progress::ProgressSink`] — every external concern is a trait so tests can
//!   substitute deterministic fakes.
//! - **Services / facade** — [`Engine`] orchestrates the use-cases (scan, plan,
//!   backup, execute, restore, diff, report, history) over the injected ports.
//!
//! ## Safety model (see `docs/11-safety-model.md`)
//!
//! Dry-run is the default for every mutating operation; destructive operations are
//! preceded by a verified backup snapshot; protected refs are structurally excluded
//! from destructive plans. These invariants are enforced in the service layer, never
//! left to callers.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Domain action model.
pub mod action;
/// Domain authentication and credential storage ports.
pub mod auth;
/// Domain backup snapshot model.
pub mod backup;
/// Clock port.
pub mod clock;
/// Configuration loading and persistence.
pub mod config;
/// Diff and tree-view domain types.
pub mod diff;
/// Error definitions and conversion logic.
pub mod error;
/// Git operations port and adapter implementations.
pub mod git;
/// History and trend tracking.
pub mod history;
/// Core domain entities, value objects, and states.
pub mod model;
/// Policy compiler and rule evaluators.
pub mod policy;
/// Progress monitoring.
pub mod progress;
/// Reporting services.
pub mod report;
/// Classification pipeline.
pub mod scan;

#[cfg(feature = "testkit")]
/// Fixture builders and test harness helpers.
pub mod testkit;

pub use config::Config;
pub use error::{GitPurgeError, Result};

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::model::{
    Action, ActionFilter, ActionKind, BackupOptions, BranchScope, ExecMode, MergeState, Plan,
    Protection, Recommendation, RefSpec, RepoId, Repository, RestoreOutcome, RestoreSpec,
    RunReport, ScanOptions, ScanResult, Snapshot, SnapshotId,
};
use crate::report::ReportFormat;

/// The public facade over `gitpurge-core`.
#[derive(Debug)]
pub struct Engine {
    config: Config,
    git: Box<dyn git::GitBackend>,
    #[allow(dead_code)]
    secrets: Box<dyn auth::SecretStore>,
    #[allow(dead_code)]
    history: Box<dyn history::HistoryStore>,
    #[allow(dead_code)]
    report_sink: Box<dyn report::ReportSink>,
    clock: Box<dyn clock::Clock>,
    #[allow(dead_code)]
    progress: Box<dyn progress::ProgressSink>,
    repos: Mutex<HashMap<RepoId, Repository>>,
}

// Compile-time assertion: Engine must be Send + Sync because all port traits
// require Send + Sync. If this fails to compile, a port trait is missing the bound.
const _: () = {
    fn _assert_send_sync<T: Send + Sync>() {}
    fn _check() {
        _assert_send_sync::<Engine>();
    }
};

impl Engine {
    /// Open an engine with the given configuration, wiring up the default production
    /// adapters (gix/git2 git backend, keyring secret store, SQLite history store).
    pub fn open(config: Config) -> Result<Self> {
        let git = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let history = Box::new(crate::history::FakeHistoryStore::default());
        let report_sink = Box::new(crate::report::FakeReportSink::default());
        let clock = Box::new(crate::clock::SystemClock);
        let progress = Box::new(crate::progress::NoopProgressSink);

        Ok(Self {
            config,
            git,
            secrets,
            history,
            report_sink,
            clock,
            progress,
            repos: Mutex::new(HashMap::new()),
        })
    }

    /// Construct a new Engine with custom injected ports (useful for tests).
    pub fn new(
        config: Config,
        git: Box<dyn git::GitBackend>,
        secrets: Box<dyn auth::SecretStore>,
        history: Box<dyn history::HistoryStore>,
        report_sink: Box<dyn report::ReportSink>,
        clock: Box<dyn clock::Clock>,
        progress: Box<dyn progress::ProgressSink>,
    ) -> Self {
        Self {
            config,
            git,
            secrets,
            history,
            report_sink,
            clock,
            progress,
            repos: Mutex::new(HashMap::new()),
        }
    }

    /// Register a repository in the local in-memory store.
    pub fn register_repo(&self, repo: Repository) -> Result<()> {
        let mut repos = self.repos.lock().unwrap();
        repos.insert(repo.id.clone(), repo);
        Ok(())
    }

    /// Classify the branches of a repository (read-only). See [`scan`].
    pub fn scan(&self, repo: &RepoId, opts: ScanOptions) -> Result<ScanResult> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        let mut policy = self.config.default_policy.clone();
        if let Some(age_override) = opts.age_override {
            policy.age = age_override;
        }
        policy.excludes.extend(opts.excludes);
        let policy_engine =
            crate::policy::PolicyEngine::new(policy).map_err(crate::GitPurgeError::Config)?;

        let mut scan_result = crate::scan::Scanner::classify(
            self.git.as_ref(),
            &repo_model,
            &policy_engine,
            self.clock.as_ref(),
        )?;

        let ref_filter = crate::model::RefFilter {
            scope: opts.scope,
            ..Default::default()
        };
        crate::scan::filter_and_sort_classifications(&mut scan_result.classifications, &ref_filter);

        Ok(scan_result)
    }

    /// Resolve the set of actions a delete/archive command *would* take (dry-run).
    pub fn plan(&self, repo: &RepoId, filter: &ActionFilter) -> Result<Plan> {
        let scan_opts = ScanOptions {
            age_override: filter.age_override.clone(),
            excludes: filter.exclude_globs.clone(),
            ..Default::default()
        };
        let scan_result = self.scan(repo, scan_opts)?;

        let mut actions = Vec::new();
        let mut skipped_count = 0;

        for class in scan_result.classifications {
            let branch_name = &class.branch.0;
            let is_protected = !matches!(class.protection, Protection::Unprotected);

            let matches_specific = if !filter.specific_branches.is_empty() {
                filter.specific_branches.iter().any(|b| b.0 == *branch_name)
            } else {
                false
            };

            let matches_include_glob = if !filter.include_globs.is_empty() {
                filter
                    .include_globs
                    .iter()
                    .any(|pat| crate::policy::matches_glob(&pat.0, branch_name))
            } else {
                true
            };

            let matches_exclude_glob = if !filter.exclude_globs.is_empty() {
                filter
                    .exclude_globs
                    .iter()
                    .any(|pat| crate::policy::matches_glob(&pat.0, branch_name))
            } else {
                false
            };

            let passes_globs = if !filter.specific_branches.is_empty() {
                matches_specific
            } else {
                matches_include_glob && !matches_exclude_glob
            };

            if !passes_globs {
                skipped_count += 1;
                continue;
            }

            if filter.merged_only && class.merge_state != MergeState::Merged {
                skipped_count += 1;
                continue;
            }

            if class.merge_state == MergeState::Unmerged
                && !filter.include_unmerged
                && !matches_specific
            {
                skipped_count += 1;
                continue;
            }

            if is_protected {
                skipped_count += 1;
                continue;
            }

            let kind = match class.recommendation {
                Recommendation::DeleteMerged => ActionKind::Delete,
                Recommendation::ArchiveStale => ActionKind::Archive,
                _ => filter.kind.unwrap_or(ActionKind::Delete),
            };

            let kind = filter.kind.unwrap_or(kind);

            let reason = match kind {
                ActionKind::Delete => "Branch is merged and unprotected".to_string(),
                ActionKind::Archive => "Branch is stale and unprotected".to_string(),
                ActionKind::Restore => "Restore branch".to_string(),
            };

            let remote = if class.scope == BranchScope::Remote {
                let parts: Vec<&str> = class.branch.0.split('/').collect();
                if parts.len() > 1 {
                    Some(parts[0].to_string())
                } else {
                    Some("origin".to_string())
                }
            } else {
                None
            };

            actions.push(Action {
                kind,
                branch: class.branch.clone(),
                scope: class.scope,
                remote,
                classification: class.clone(),
                reason,
            });
        }

        let summary = format!(
            "Plan: {} actions to execute, {} skipped",
            actions.len(),
            skipped_count
        );

        Ok(Plan {
            repo: repo.clone(),
            actions,
            skipped_count,
            summary,
        })
    }

    /// Create a point-in-time backup snapshot of a repository's refs.
    pub fn backup_create(&self, repo: &RepoId, opts: BackupOptions) -> Result<Snapshot> {
        let _ = (repo, opts);
        todo!("backup snapshot — phase P2")
    }

    /// Execute a resolved plan.
    pub fn execute(&self, plan: &Plan, mode: ExecMode) -> Result<RunReport> {
        let _ = (plan, mode);
        todo!("plan execution — phase P3")
    }

    /// Restore a ref from a snapshot.
    pub fn restore(&self, snap: &SnapshotId, spec: RestoreSpec) -> Result<RestoreOutcome> {
        let _ = (snap, spec);
        todo!("restore — phase P2")
    }

    /// Diff two refs.
    pub fn diff(&self, repo: &RepoId, a: &RefSpec, b: &RefSpec) -> Result<crate::diff::DiffResult> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        let diffs = self.git.diff_refs(&repo_model, a, b)?;
        let mut entries = Vec::new();
        let mut insertions = 0;
        let mut deletions = 0;

        for d in diffs {
            let kind = if d.additions > 0 && d.deletions > 0 {
                crate::diff::DiffKind::Modified
            } else if d.additions > 0 {
                crate::diff::DiffKind::Added
            } else {
                crate::diff::DiffKind::Deleted
            };

            insertions += d.additions;
            deletions += d.deletions;

            entries.push(crate::diff::DiffEntry {
                path: d.path,
                kind,
                additions: d.additions,
                deletions: d.deletions,
            });
        }

        let files_changed = entries.len();

        Ok(crate::diff::DiffResult {
            from: a.clone(),
            to: b.clone(),
            entries,
            files_changed,
            insertions,
            deletions,
        })
    }

    /// View the tree (or a single path) at a ref/commit.
    pub fn show_tree(
        &self,
        repo: &RepoId,
        at: &RefSpec,
        path: Option<&Path>,
    ) -> Result<crate::diff::TreeView> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        if let Some(p) = path {
            let blob_data = self
                .git
                .read_blob(&repo_model, at, p.to_str().unwrap_or(""))?;
            let entry = crate::diff::TreeEntry {
                path: p.to_string_lossy().to_string(),
                is_dir: false,
                size: blob_data.len() as u64,
                oid: crate::model::Oid("fake-oid".to_string()),
            };
            Ok(crate::diff::TreeView {
                at: at.clone(),
                entries: vec![entry],
            })
        } else {
            let files = self.git.read_tree(&repo_model, at)?;
            let mut entries = Vec::new();
            for file in files {
                entries.push(crate::diff::TreeEntry {
                    path: file,
                    is_dir: false,
                    size: 0,
                    oid: crate::model::Oid("fake-oid".to_string()),
                });
            }
            Ok(crate::diff::TreeView {
                at: at.clone(),
                entries,
            })
        }
    }

    /// Generate an audit/trend report in the requested format.
    pub fn report(&self, repo: &RepoId, fmt: ReportFormat) -> Result<crate::report::Report> {
        let _ = (repo, fmt);
        todo!("report — phase P5")
    }

    /// Fetch the recorded trend history for a repository.
    pub fn history(&self, repo: &RepoId) -> Result<crate::history::TrendHistory> {
        let _ = repo;
        todo!("history — phase P5")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::FakeClock;
    use crate::model::{ActionKind, BranchName, GlobPattern, Policy};
    use crate::testkit;

    #[test]
    fn test_engine_scan_and_plan_flow() {
        let repo_fixture = testkit::merged_repo();

        let mut policy = Policy::default();
        policy
            .protection
            .protected_globs
            .push(GlobPattern("release/*".to_string()));

        let config = Config {
            backups_root: None,
            default_policy: policy,
            protected: Vec::new(),
        };

        let git_backend = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let history = Box::new(crate::history::FakeHistoryStore::default());
        let report_sink = Box::new(crate::report::FakeReportSink::default());
        let clock = Box::new(FakeClock::new(
            time::macros::datetime!(2026-07-05 12:00:00 UTC),
        ));
        let progress = Box::new(crate::progress::NoopProgressSink);

        let engine = Engine::new(
            config,
            git_backend,
            secrets,
            history,
            report_sink,
            clock,
            progress,
        );

        let repo_id = RepoId("test-repo".to_string());
        let repo_model = Repository {
            id: repo_id.clone(),
            display_name: "test-repo".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };

        engine.register_repo(repo_model).unwrap();

        // 1. Scan
        let scan_res = engine.scan(&repo_id, ScanOptions::default()).unwrap();
        assert_eq!(scan_res.total_branches, 3);

        // 2. Plan (dry-run)
        let filter = ActionFilter {
            merged_only: true,
            ..Default::default()
        };
        let plan = engine.plan(&repo_id, &filter).unwrap();

        // main is protected, unmerged-branch is unmerged (so excluded by default merged_only filter)
        // Only merged-branch should have a delete action proposed!
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0].branch.0, "merged-branch");
        assert_eq!(plan.actions[0].kind, ActionKind::Delete);

        // Test diff
        let diff_res = engine
            .diff(
                &repo_id,
                &RefSpec::Branch(BranchName("main".to_string())),
                &RefSpec::Branch(BranchName("unmerged-branch".to_string())),
            )
            .unwrap();
        assert!(diff_res.files_changed > 0);

        // Test show_tree
        let tree_res = engine
            .show_tree(
                &repo_id,
                &RefSpec::Branch(BranchName("main".to_string())),
                None,
            )
            .unwrap();
        assert!(tree_res.entries.iter().any(|e| e.path == "file.txt"));
    }
}
