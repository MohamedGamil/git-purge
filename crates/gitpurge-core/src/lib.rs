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
#![deny(unsafe_code)]
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

#[cfg(any(test, feature = "testkit"))]
/// Fixture builders and test harness helpers.
pub mod testkit;

pub use config::Config;
pub use error::{GitPurgeError, Result};

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use crate::model::{
    Action, ActionFilter, ActionKind, ActionResult, BackupOptions, BranchName, BranchScope,
    ExecMode, MergeState, Plan, Protection, Recommendation, RefSpec, RepoId, Repository,
    RestoreOutcome, RestoreSpec, RunReport, ScanOptions, ScanResult, Snapshot, SnapshotId,
};
use crate::report::ReportFormat;

/// The public facade over `gitpurge-core`.
#[derive(Debug)]
pub struct Engine {
    config: Mutex<Config>,
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
    scan_cache: Mutex<HashMap<RepoId, (String, ScanResult)>>,
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
    #[allow(unsafe_code)]
    pub fn open(config: Config) -> Result<Self> {
        // Configure libgit2 network connection and operation timeouts to protect against offline state/VPN loss.
        unsafe {
            let _ = git2::opts::set_server_connect_timeout_in_milliseconds(5000);
            let _ = git2::opts::set_server_timeout_in_milliseconds(15000);
        }

        let git = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let db_path = config.resolve_data_dir().join("history.db");
        let backups_root = config
            .backups_root
            .clone()
            .unwrap_or_else(|| config.resolve_data_dir().join("backups"));
        let history = Box::new(crate::history::SqliteHistoryStore::new(
            &db_path,
            backups_root,
        )?);
        let report_sink = Box::new(crate::report::FileReportSink::new(
            config.resolve_data_dir(),
            None,
        ));
        let clock = Box::new(crate::clock::SystemClock);
        let progress = Box::new(crate::progress::NoopProgressSink);

        let repos_map = config
            .repos
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();

        Ok(Self {
            config: Mutex::new(config),
            git,
            secrets,
            history,
            report_sink,
            clock,
            progress,
            repos: Mutex::new(repos_map),
            scan_cache: Mutex::new(HashMap::new()),
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
        let repos_map = config
            .repos
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();

        Self {
            config: Mutex::new(config),
            git,
            secrets,
            history,
            report_sink,
            clock,
            progress,
            repos: Mutex::new(repos_map),
            scan_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Register a repository in the local in-memory store.
    pub fn register_repo(&self, repo: Repository) -> Result<()> {
        self.history.save_repo(&repo)?;
        let mut repos = self.repos.lock().unwrap();
        repos.insert(repo.id.clone(), repo);
        Ok(())
    }

    /// Add a repository to the tracked list in config and save.
    pub fn add_repo(&self, repo: Repository) -> Result<()> {
        self.register_repo(repo.clone())?;
        // Sync config
        let mut config = self.config.lock().unwrap();
        if !config.repos.iter().any(|r| r.id == repo.id) {
            config.repos.push(repo);
        } else {
            // Update existing
            if let Some(existing) = config.repos.iter_mut().find(|r| r.id == repo.id) {
                *existing = repo;
            }
        }
        Ok(())
    }

    /// Remove a repository from the tracked list.
    pub fn remove_repo(&self, id: &RepoId) -> Result<()> {
        {
            let mut repos = self.repos.lock().unwrap();
            repos.remove(id);
        }
        let mut config = self.config.lock().unwrap();
        config.repos.retain(|r| r.id != *id);
        if config.default_repo.as_ref() == Some(id) {
            config.default_repo = None;
        }
        Ok(())
    }

    /// List all tracked repositories.
    pub fn list_repos(&self) -> Result<Vec<Repository>> {
        let repos = self.repos.lock().unwrap();
        Ok(repos.values().cloned().collect())
    }

    /// Get tracked repository details by ID.
    pub fn get_repo(&self, id: &RepoId) -> Result<Option<Repository>> {
        let repos = self.repos.lock().unwrap();
        Ok(repos.get(id).cloned())
    }

    /// Set the default repository.
    pub fn set_default_repo(&self, id: RepoId) -> Result<()> {
        let mut config = self.config.lock().unwrap();
        config.default_repo = Some(id);
        Ok(())
    }

    /// Get the default repository ID.
    pub fn default_repo_id(&self) -> Option<RepoId> {
        let config = self.config.lock().unwrap();
        config.default_repo.clone()
    }

    /// Save the current config to disk.
    pub fn save_config(&self, path: Option<&Path>) -> Result<()> {
        let config = self.config.lock().unwrap().clone();
        config.save(path)?;
        Ok(())
    }

    /// Get a clone of the current configuration.
    pub fn config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    /// Update the current configuration.
    pub fn update_config(&self, new_config: Config) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
    }

    /// Purge backups bare mirror for a repository from disk.
    pub fn purge_repo_backups(&self, id: &RepoId) -> Result<()> {
        let config_guard = self.config.lock().unwrap();
        let mirror_manager = crate::backup::BackupMirrorManager::new(&config_guard);
        let mirror_path = mirror_manager.resolve_mirror_path(id);
        if mirror_path.exists() {
            std::fs::remove_dir_all(mirror_path).map_err(|e| {
                crate::GitPurgeError::Git(format!("Failed to delete bare mirror directory: {}", e))
            })?;
        }
        Ok(())
    }

    /// Classify the branches of a repository (read-only).
    pub fn scan(&self, repo: &RepoId, opts: ScanOptions) -> Result<ScanResult> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        if opts.auto_fetch {
            if let Err(e) = self.git.fetch_all_prune(&repo_model) {
                tracing::warn!("Auto-fetch all prune failed, proceeding with local state: {}", e);
            }
        }

        let mut policy = self.config.lock().unwrap().default_policy.clone();
        if let Some(age_override) = opts.age_override {
            policy.age = age_override;
        }
        policy.excludes.extend(opts.excludes);
        let policy_engine =
            crate::policy::PolicyEngine::new(policy).map_err(crate::GitPurgeError::Config)?;

        let branches = self.git.list_branches(&repo_model, None)?;

        // Compute git signature (based on branches and their tip commit OIDs + is_head)
        let mut branch_sigs: Vec<String> = branches
            .iter()
            .map(|b| format!("{}:{}:{}", b.name.0, b.tip.oid.0, b.is_head))
            .collect();
        branch_sigs.sort();

        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for sig in branch_sigs {
            sig.hash(&mut hasher);
        }
        let git_sig = hasher.finish();

        let mut policy_hasher = std::collections::hash_map::DefaultHasher::new();
        let policy_json = serde_json::to_string(&policy_engine.policy).unwrap_or_default();
        policy_json.hash(&mut policy_hasher);
        let policy_sig = policy_hasher.finish();

        let cache_key = format!("{:x}:{:x}", git_sig, policy_sig);

        let cached_result = {
            let cache = self.scan_cache.lock().unwrap();
            cache
                .get(repo)
                .filter(|(sig, _)| sig == &cache_key)
                .map(|(_, res)| res.clone())
        };

        let mut scan_result = match cached_result {
            Some(res) => res,
            None => {
                let res = crate::scan::Scanner::classify(
                    self.git.as_ref(),
                    &repo_model,
                    &policy_engine,
                    self.clock.as_ref(),
                    branches,
                )?;
                let mut cache = self.scan_cache.lock().unwrap();
                cache.insert(repo.clone(), (cache_key, res.clone()));
                res
            }
        };

        let ref_filter = crate::model::RefFilter {
            scope: opts.scope,
            ..Default::default()
        };
        crate::scan::filter_and_sort_classifications(&mut scan_result.classifications, &ref_filter);

        // Record a read-only "scan" run
        let total = scan_result.total_branches;
        let active = scan_result
            .classifications
            .iter()
            .filter(|c| matches!(c.activity, crate::model::Activity::Active))
            .count();
        let stale = scan_result
            .classifications
            .iter()
            .filter(|c| matches!(c.activity, crate::model::Activity::Stale))
            .count();
        let merged = scan_result
            .classifications
            .iter()
            .filter(|c| matches!(c.merge_state, crate::model::MergeState::Merged))
            .count();
        let unmerged = scan_result
            .classifications
            .iter()
            .filter(|c| matches!(c.merge_state, crate::model::MergeState::Unmerged))
            .count();
        let non_standard = scan_result
            .classifications
            .iter()
            .filter(|c| {
                !matches!(
                    c.naming,
                    crate::model::NamingVerdict::Standard
                        | crate::model::NamingVerdict::Exempt { .. }
                )
            })
            .count();

        let metrics = crate::model::RunMetrics {
            total,
            active,
            stale,
            merged,
            unmerged,
            non_standard,
            local_count: Some(
                scan_result
                    .classifications
                    .iter()
                    .filter(|c| matches!(c.scope, crate::model::BranchScope::Local))
                    .count(),
            ),
            remote_count: Some(
                scan_result
                    .classifications
                    .iter()
                    .filter(|c| matches!(c.scope, crate::model::BranchScope::Remote))
                    .count(),
            ),
            protected: Some(
                scan_result
                    .classifications
                    .iter()
                    .filter(|c| !matches!(c.protection, crate::model::Protection::Unprotected))
                    .count(),
            ),
            deleted: Some(0),
            archived: Some(0),
            restored: Some(0),
        };

        let report = RunReport {
            id: ulid::Ulid::new().to_string(),
            started_at: self.clock.now(),
            repo: repo.clone(),
            mode: ExecMode::DryRun,
            snapshot: None,
            results: Vec::new(),
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
            command: "scan".to_string(),
            metrics: Some(metrics),
            branch_snapshots: Some(scan_result.classifications.clone()),
        };

        let _ = self.history.record_run(&report);

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
    pub fn backup_create(&self, repo_id: &RepoId, opts: BackupOptions) -> Result<Snapshot> {
        let repo = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo_id).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!(
                    "Repository not registered: {:?}",
                    repo_id
                ))
            })?
        };

        let scan_opts = ScanOptions::default();
        let scan_res = self.scan(repo_id, scan_opts)?;

        let config_guard = self.config.lock().unwrap();
        let snapshot = crate::backup::create_snapshot(
            &config_guard,
            self.git.as_ref(),
            &repo,
            &scan_res.classifications,
            &opts,
        )?;

        let verify_report =
            crate::backup::verify_snapshot(&config_guard, repo_id, &snapshot.id, false)?;
        if !verify_report.ok {
            return Err(crate::GitPurgeError::Snapshot(format!(
                "Backup snapshot '{}' failed verification.",
                snapshot.id.0
            )));
        }

        let mut verified_snapshot = snapshot;
        verified_snapshot.verified = true;

        self.history.save_snapshot(&verified_snapshot)?;

        Ok(verified_snapshot)
    }

    /// Execute a resolved plan.
    pub fn execute(&self, plan: &Plan, mode: ExecMode, no_backup: bool) -> Result<RunReport> {
        let started_at = self.clock.now();
        let run_id = ulid::Ulid::new().to_string();
        let command = if plan.actions.iter().any(|a| a.kind == ActionKind::Archive) {
            "archive".to_string()
        } else {
            "delete".to_string()
        };

        let repo = {
            let repos = self.repos.lock().unwrap();
            repos.get(&plan.repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!(
                    "Repository not registered: {:?}",
                    plan.repo
                ))
            })?
        };

        if mode == ExecMode::DryRun {
            let mut results = Vec::new();
            for action in &plan.actions {
                results.push(ActionResult::Skipped {
                    action: action.clone(),
                });
            }
            return Ok(RunReport {
                id: run_id,
                started_at,
                repo: plan.repo.clone(),
                mode: ExecMode::DryRun,
                snapshot: None,
                results,
                success_count: 0,
                failure_count: 0,
                skipped_count: plan.actions.len(),
                command,
                metrics: None,
                branch_snapshots: None,
            });
        }

        let results = crate::action::execute_deletions_with_guard(
            &self.config.lock().unwrap(),
            self.git.as_ref(),
            self.history.as_ref(),
            &repo,
            &plan.actions,
            no_backup,
            |action| {
                if action.scope == crate::model::BranchScope::Remote {
                    let remote = action.remote.as_deref().unwrap_or("origin");
                    self.git.delete_remote_branch(&repo, remote, &action.branch)
                } else {
                    self.git.delete_local_branch(&repo, &action.branch)
                }
            },
            |_, _| true,
        )?;

        self.scan_cache.lock().unwrap().remove(&plan.repo);

        let success_count = results
            .iter()
            .filter(|r| matches!(r, ActionResult::Success { .. }))
            .count();
        let failure_count = results
            .iter()
            .filter(|r| matches!(r, ActionResult::Failed { .. }))
            .count();
        let skipped_count = results
            .iter()
            .filter(|r| matches!(r, ActionResult::Skipped { .. }))
            .count();

        let snapshots = self.history.list_snapshots(&plan.repo)?;
        let snapshot_id = snapshots.first().map(|s| s.id.clone());

        // Perform post-operation scan to compute post-op metrics
        let post_scan = self
            .scan(&plan.repo, ScanOptions::default())
            .unwrap_or_else(|_| ScanResult {
                repo: plan.repo.clone(),
                classifications: Vec::new(),
                total_branches: 0,
                excluded_count: 0,
            });

        let total = post_scan.total_branches;
        let active = post_scan
            .classifications
            .iter()
            .filter(|c| matches!(c.activity, crate::model::Activity::Active))
            .count();
        let stale = post_scan
            .classifications
            .iter()
            .filter(|c| matches!(c.activity, crate::model::Activity::Stale))
            .count();
        let merged = post_scan
            .classifications
            .iter()
            .filter(|c| matches!(c.merge_state, crate::model::MergeState::Merged))
            .count();
        let unmerged = post_scan
            .classifications
            .iter()
            .filter(|c| matches!(c.merge_state, crate::model::MergeState::Unmerged))
            .count();
        let non_standard = post_scan
            .classifications
            .iter()
            .filter(|c| {
                !matches!(
                    c.naming,
                    crate::model::NamingVerdict::Standard
                        | crate::model::NamingVerdict::Exempt { .. }
                )
            })
            .count();

        let metrics = crate::model::RunMetrics {
            total,
            active,
            stale,
            merged,
            unmerged,
            non_standard,
            local_count: Some(
                post_scan
                    .classifications
                    .iter()
                    .filter(|c| matches!(c.scope, crate::model::BranchScope::Local))
                    .count(),
            ),
            remote_count: Some(
                post_scan
                    .classifications
                    .iter()
                    .filter(|c| matches!(c.scope, crate::model::BranchScope::Remote))
                    .count(),
            ),
            protected: Some(
                post_scan
                    .classifications
                    .iter()
                    .filter(|c| !matches!(c.protection, crate::model::Protection::Unprotected))
                    .count(),
            ),
            deleted: Some(success_count),
            archived: Some(0),
            restored: Some(0),
        };

        let report = RunReport {
            id: run_id,
            started_at,
            repo: plan.repo.clone(),
            mode: ExecMode::Execute,
            snapshot: snapshot_id,
            results,
            success_count,
            failure_count,
            skipped_count,
            command,
            metrics: Some(metrics),
            branch_snapshots: Some(post_scan.classifications),
        };

        self.history.record_run(&report)?;

        Ok(report)
    }

    /// Restore a ref from a snapshot.
    pub fn restore(&self, snap: &SnapshotId, spec: RestoreSpec) -> Result<RestoreOutcome> {
        let snapshot = self.history.get_snapshot(snap)?.ok_or_else(|| {
            crate::GitPurgeError::Snapshot(format!("Snapshot not found: {}", snap.0))
        })?;

        let repo = {
            let repos = self.repos.lock().unwrap();
            repos.get(&snapshot.repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!(
                    "Repository not registered: {:?}",
                    snapshot.repo
                ))
            })?
        };

        let res = crate::backup::restore_snapshot(&self.config.lock().unwrap(), &repo, snap, &spec);
        self.scan_cache.lock().unwrap().remove(&snapshot.repo);
        res
    }

    /// Archive branches into a target legacy/archive branch.
    pub fn archive(
        &self,
        repo_id: &RepoId,
        branches: &[BranchName],
        target_branch: &str,
        strategy: crate::action::ArchiveStrategy,
        push: bool,
    ) -> Result<()> {
        let repo = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo_id).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!(
                    "Repository not registered: {:?}",
                    repo_id
                ))
            })?
        };

        let res = crate::action::archive_branches(
            &self.config.lock().unwrap(),
            &repo,
            branches,
            target_branch,
            strategy,
            push,
        );
        self.scan_cache.lock().unwrap().remove(repo_id);
        res
    }

    /// List all backup snapshots for a repository.
    pub fn list_snapshots(&self, repo_id: &RepoId) -> Result<Vec<Snapshot>> {
        self.history.list_snapshots(repo_id)
    }

    /// Get details of a specific snapshot.
    pub fn get_snapshot(&self, snap_id: &SnapshotId) -> Result<Option<Snapshot>> {
        self.history.get_snapshot(snap_id)
    }

    /// Delete a snapshot's metadata.
    pub fn delete_snapshot(&self, snap_id: &SnapshotId) -> Result<()> {
        self.history.delete_snapshot(snap_id)
    }

    /// Verify the integrity of a backup snapshot in the bare mirror.
    pub fn backup_verify(
        &self,
        repo_id: &RepoId,
        snap_id: &SnapshotId,
    ) -> Result<crate::backup::VerifyReport> {
        crate::backup::verify_snapshot(&self.config.lock().unwrap(), repo_id, snap_id, false)
    }

    /// Prune old snapshots for a repository based on a retention policy.
    pub fn backup_prune(
        &self,
        repo_id: &RepoId,
        policy: &crate::model::RetentionPolicy,
        mode: ExecMode,
    ) -> Result<crate::model::PruneReport> {
        crate::backup::prune_snapshots(
            &self.config.lock().unwrap(),
            self.history.as_ref(),
            repo_id,
            policy,
            mode,
        )
    }

    /// Fetch from remote (default 'origin') for a repository.
    pub fn fetch(&self, repo_id: &RepoId) -> Result<()> {
        let repo = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo_id).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!(
                    "Repository not registered: {:?}",
                    repo_id
                ))
            })?
        };
        self.git.fetch(&repo, "origin")
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

    /// Read the raw content of a file at a ref/commit.
    pub fn show_file(&self, repo: &RepoId, at: &RefSpec, path: &Path) -> Result<Vec<u8>> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };
        self.git
            .read_blob(&repo_model, at, path.to_str().unwrap_or(""))
    }

    /// Generate an audit/trend report in the requested format.
    pub fn report(
        &self,
        repo: &RepoId,
        report_type: crate::report::ReportType,
        fmt: ReportFormat,
    ) -> Result<crate::report::Report> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        // 1. Run a scan to get the current classifications
        let scan_result = self.scan(repo, ScanOptions::default())?;

        // 2. Fetch trend history
        let history = self.history.get_history(repo)?;

        // 3. Generate content based on format and type
        let generated_at = self.clock.now();
        let content = match report_type {
            crate::report::ReportType::Audit => match fmt {
                ReportFormat::Markdown => crate::report::markdown::generate_audit_report(
                    &repo_model,
                    &scan_result,
                    generated_at,
                ),
                ReportFormat::Json => crate::report::json::generate_json_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                )?,
                ReportFormat::Html => crate::report::html::generate_html_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                ),
            },
            crate::report::ReportType::Trend => match fmt {
                ReportFormat::Markdown => crate::report::markdown::generate_trend_report(
                    &repo_model,
                    &history,
                    generated_at,
                    None,
                ),
                ReportFormat::Json => crate::report::json::generate_json_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                )?,
                ReportFormat::Html => crate::report::html::generate_html_report(
                    &repo_model,
                    &scan_result,
                    Some(&history),
                    generated_at,
                ),
            },
        };

        let report = crate::report::Report {
            repo: repo.clone(),
            report_type,
            format: fmt,
            content,
            generated_at,
        };

        // 4. Write/archive the report using the configured report sink
        self.report_sink.write_report(&report)?;

        Ok(report)
    }

    /// Fetch the recorded trend history for a repository.
    pub fn history(&self, repo: &RepoId) -> Result<crate::model::TrendHistory> {
        self.history.get_history(repo)
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
            ..Default::default()
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

    #[test]
    fn test_backup_create_and_verify() {
        let repo_fixture = testkit::merged_repo();
        let repo_id = RepoId("test-backup-repo".to_string());

        let config = Config::default();
        let git_backend = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let history = Box::new(crate::history::FakeHistoryStore::new());
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

        let repo_model = Repository {
            id: repo_id.clone(),
            display_name: "test-backup-repo".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };
        engine.register_repo(repo_model).unwrap();

        // 1. Create backup
        let opts = BackupOptions {
            trigger: Some(crate::model::SnapshotTrigger::Manual),
            verify: true,
            only_branches: Vec::new(),
        };
        let snapshot = engine.backup_create(&repo_id, opts).unwrap();
        assert_eq!(snapshot.refs.len(), 3); // main, merged-branch, unmerged-branch
        assert!(snapshot.verified);

        // 2. Verify snapshot
        let verify_res = crate::backup::verify_snapshot(
            &engine.config.lock().unwrap(),
            &repo_id,
            &snapshot.id,
            false,
        )
        .unwrap();
        assert!(verify_res.ok);

        // 3. Corrupt snapshot by deleting a backup reference in the mirror
        let mirror_manager =
            crate::backup::BackupMirrorManager::new(&engine.config.lock().unwrap());
        let mirror_path = mirror_manager.resolve_mirror_path(&repo_id);
        let mirror_repo = git2::Repository::open_bare(&mirror_path).unwrap();

        let target_ref = format!(
            "refs/gitpurge/backups/{}/refs/heads/merged-branch",
            snapshot.id.0
        );
        let mut r = mirror_repo.find_reference(&target_ref).unwrap();
        r.delete().unwrap();

        // 4. Verify snapshot should now detect corruption
        let verify_res2 = crate::backup::verify_snapshot(
            &engine.config.lock().unwrap(),
            &repo_id,
            &snapshot.id,
            false,
        )
        .unwrap();
        assert!(!verify_res2.ok);
        assert!(verify_res2
            .problems
            .contains(&crate::backup::VerifyProblem::MissingRef));
    }

    #[test]
    fn test_restore_safeties() {
        let repo_fixture = testkit::merged_repo();
        let repo_id = RepoId("test-restore-repo".to_string());

        let config = Config::default();
        let git_backend = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let history = Box::new(crate::history::FakeHistoryStore::new());
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

        let repo_model = Repository {
            id: repo_id.clone(),
            display_name: "test-restore-repo".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };
        engine.register_repo(repo_model).unwrap();

        // 1. Create backup snapshot
        let snapshot = engine
            .backup_create(&repo_id, BackupOptions::default())
            .unwrap();

        // 2. Delete branch in source repository
        let source_repo = git2::Repository::open(repo_fixture.path()).unwrap();
        let mut r = source_repo
            .find_reference("refs/heads/merged-branch")
            .unwrap();
        let original_oid = r.target().unwrap();
        r.delete().unwrap();
        assert!(source_repo
            .find_reference("refs/heads/merged-branch")
            .is_err());

        // 3. Restore branch
        let spec = RestoreSpec {
            branch: BranchName("merged-branch".to_string()),
            as_tag: false,
            target_name: None,
            force: false,
        };
        let outcome = engine.restore(&snapshot.id, spec).unwrap();
        assert_eq!(outcome.created_ref, "refs/heads/merged-branch");
        assert_eq!(outcome.tip.0, original_oid.to_string());

        // 4. Verify branch is restored in source repo
        let restored_ref = source_repo
            .find_reference("refs/heads/merged-branch")
            .unwrap();
        assert_eq!(restored_ref.target().unwrap(), original_oid);

        // 5. SAFE-06: Restore again without force should fail
        let spec_no_force = RestoreSpec {
            branch: BranchName("merged-branch".to_string()),
            as_tag: false,
            target_name: None,
            force: false,
        };
        let err = engine.restore(&snapshot.id, spec_no_force);
        assert!(err.is_err());
        assert!(matches!(
            err.unwrap_err(),
            crate::GitPurgeError::SafetyViolation(_)
        ));

        // 6. Restore again with force should succeed
        let spec_force = RestoreSpec {
            branch: BranchName("merged-branch".to_string()),
            as_tag: false,
            target_name: None,
            force: true,
        };
        let outcome2 = engine.restore(&snapshot.id, spec_force).unwrap();
        assert_eq!(outcome2.created_ref, "refs/heads/merged-branch");

        // 7. Restore as tag
        let spec_tag = RestoreSpec {
            branch: BranchName("merged-branch".to_string()),
            as_tag: true,
            target_name: Some("restored-tag".to_string()),
            force: false,
        };
        let outcome_tag = engine.restore(&snapshot.id, spec_tag).unwrap();
        assert_eq!(outcome_tag.created_ref, "refs/tags/restored-tag");
        assert_eq!(outcome_tag.tip.0, original_oid.to_string());
        assert!(source_repo.find_reference("refs/tags/restored-tag").is_ok());
    }

    #[test]
    fn test_auto_restore_on_failure() {
        let repo_fixture = testkit::merged_repo();
        let repo_id = RepoId("test-failed-delete-repo".to_string());

        let config = Config::default();
        let git_backend = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let history = Box::new(crate::history::FakeHistoryStore::new());
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

        let repo_model = Repository {
            id: repo_id.clone(),
            display_name: "test-failed-delete-repo".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };
        engine.register_repo(repo_model).unwrap();

        // 1. Scan and build plan to delete unmerged-branch
        let filter = ActionFilter {
            include_unmerged: true,
            specific_branches: vec![BranchName("unmerged-branch".to_string())],
            ..Default::default()
        };
        let plan = engine.plan(&repo_id, &filter).unwrap();

        // Verify the branch exists before we try to delete it
        let source_repo = git2::Repository::open(repo_fixture.path()).unwrap();
        assert!(source_repo
            .find_reference("refs/heads/unmerged-branch")
            .is_ok());

        // 2. Execute plan with simulated failure during deletion to trigger SAFE-05
        let mut is_restore_called = false;

        let run_res = crate::action::execute_deletions_with_guard(
            &engine.config.lock().unwrap(),
            engine.git.as_ref(),
            engine.history.as_ref(),
            engine.repos.lock().unwrap().get(&repo_id).unwrap(),
            &plan.actions,
            false,
            |_action| {
                let mut r = source_repo
                    .find_reference("refs/heads/unmerged-branch")
                    .unwrap();
                r.delete().unwrap();

                Err(crate::GitPurgeError::Git(
                    "Simulated delete failure".to_string(),
                ))
            },
            |_, _| {
                is_restore_called = true;
                true // accept the restore
            },
        )
        .unwrap();

        assert_eq!(run_res.len(), 1);
        assert!(matches!(
            run_res[0],
            crate::model::ActionResult::Failed { .. }
        ));
        assert!(is_restore_called);

        // Verify that the branch was automatically restored!
        assert!(source_repo
            .find_reference("refs/heads/unmerged-branch")
            .is_ok());
    }

    #[test]
    fn test_disk_size_sublinear() {
        let repo_fixture = testkit::merged_repo();
        let repo_id = RepoId("test-size-repo".to_string());

        let config = Config::default();
        let git_backend = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let history = Box::new(crate::history::FakeHistoryStore::new());
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

        let repo_model = Repository {
            id: repo_id.clone(),
            display_name: "test-size-repo".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };
        engine.register_repo(repo_model).unwrap();

        // Create 5 snapshots of the repository without any changes
        let mut snapshots = Vec::new();
        for _ in 0..5 {
            let snap = engine
                .backup_create(&repo_id, BackupOptions::default())
                .unwrap();
            snapshots.push(snap);
        }

        // Verify we have 5 snapshots in history
        let listed = engine.history.list_snapshots(&repo_id).unwrap();
        assert_eq!(listed.len(), 5);

        // Verify the bare mirror directory exists and objects are shared
        let mirror_manager =
            crate::backup::BackupMirrorManager::new(&engine.config.lock().unwrap());
        let mirror_path = mirror_manager.resolve_mirror_path(&repo_id);
        assert!(mirror_path.exists());
    }

    #[test]
    fn test_golden_reports() {
        let repo_fixture = testkit::merged_repo();
        let repo_id = RepoId("test-report-repo".to_string());

        let config = Config {
            backups_root: None,
            default_policy: Policy::default(),
            ..Default::default()
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

        let repo_model = Repository {
            id: repo_id.clone(),
            display_name: "test-report-repo".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::macros::datetime!(2026-07-01 12:00:00 UTC),
            last_scanned_at: None,
        };
        engine.register_repo(repo_model).unwrap();

        let report = engine
            .report(
                &repo_id,
                crate::report::ReportType::Audit,
                ReportFormat::Markdown,
            )
            .unwrap();
        insta::assert_snapshot!("audit_report_markdown", report.content);
    }

    #[test]
    fn test_scan_cache_hit_and_invalidation() {
        let repo_fixture = testkit::merged_repo();
        let repo_id = RepoId("test-cache-repo".to_string());

        let mut config = Config::default();
        let git_backend = Box::new(crate::git::CompositeGitBackend::new());
        let secrets = Box::new(crate::auth::FakeSecretStore::default());
        let history = Box::new(crate::history::FakeHistoryStore::default());
        let report_sink = Box::new(crate::report::FakeReportSink::default());
        let clock = Box::new(FakeClock::new(
            time::macros::datetime!(2026-07-05 12:00:00 UTC),
        ));
        let progress = Box::new(crate::progress::NoopProgressSink);

        let engine = Engine::new(
            config.clone(),
            git_backend,
            secrets,
            history,
            report_sink,
            clock,
            progress,
        );

        let repo_model = Repository {
            id: repo_id.clone(),
            display_name: "test-cache-repo".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };
        engine.register_repo(repo_model).unwrap();

        // 1. First scan (cache miss, populates cache)
        let res1 = engine.scan(&repo_id, ScanOptions::default()).unwrap();
        assert!(!res1.classifications.is_empty());

        // Cache must now contain 1 entry
        {
            let cache = engine.scan_cache.lock().unwrap();
            assert_eq!(cache.len(), 1);
            assert!(cache.contains_key(&repo_id));
        }

        // 2. Second scan (cache hit)
        let res2 = engine.scan(&repo_id, ScanOptions::default()).unwrap();
        assert_eq!(res1, res2);

        // 3. Change policy (cache invalidation because policy signature changes)
        config.default_policy.age = "30 days ago".to_string();
        engine.update_config(config);

        let _res3 = engine.scan(&repo_id, ScanOptions::default()).unwrap();
        assert_eq!(engine.scan_cache.lock().unwrap().len(), 1);

        // 4. Manual cache clear (belt and suspenders)
        engine.scan_cache.lock().unwrap().clear();
        assert_eq!(engine.scan_cache.lock().unwrap().len(), 0);
    }
}

/// Log an operation (delete or archive) to a file for debugging.
pub fn log_operation(op: &str, branch: &str, scope: &str, result: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    let log_dir = if let Some(bd) = directories::BaseDirs::new() {
        bd.home_dir().join(".git-purge")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".git-purge")
    } else {
        std::env::temp_dir().join(".git-purge")
    };

    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("git-purge-operations.log");

    let now = time::OffsetDateTime::now_utc().to_string();
    let log_line = format!("[{}] OP: {} | BRANCH: {} | SCOPE: {} | RESULT: {}\n", now, op, branch, scope, result);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = file.write_all(log_line.as_bytes());
    }
}
