//! Action execution & restore methods (P10-T2).

use crate::error::Result;
use crate::model::{
    ActionKind, ActionResult, BranchName, ExecMode, Plan, RepoId, RestoreOutcome, RestoreSpec,
    RunMetrics, RunReport, ScanOptions, ScanResult, SnapshotId,
};

impl super::Engine {
    /// Execute a resolved plan.
    pub fn execute(&self, plan: &Plan, mode: ExecMode, no_backup: bool) -> Result<RunReport> {
        self.execute_with_progress(plan, mode, no_backup, self.progress.as_ref())
    }

    /// Execute a resolved plan with a custom progress sink.
    pub fn execute_with_progress(
        &self,
        plan: &Plan,
        mode: ExecMode,
        no_backup: bool,
        progress: &dyn crate::progress::ProgressSink,
    ) -> Result<RunReport> {
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

        let action_name = if command == "archive" {
            "Archiving"
        } else {
            "Deleting"
        };
        let results = crate::action::delete_branches(
            &self.config.lock().unwrap(),
            self.git.as_ref(),
            self.history.as_ref(),
            &repo,
            &plan.actions,
            no_backup,
            progress,
            action_name,
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

        let metrics = RunMetrics {
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
}
