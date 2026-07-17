//! Scan & Plan methods (P10-T2).

use crate::error::Result;
use crate::model::{
    Action, ActionFilter, ActionKind, BranchScope, ExecMode, MergeState, Plan, Protection,
    Recommendation, RepoId, RunMetrics, RunReport, ScanOptions, ScanResult,
};

impl super::Engine {
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
                tracing::warn!(
                    "Auto-fetch all prune failed, proceeding with local state: {}",
                    e
                );
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

        let metrics = RunMetrics {
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
                class.remote.clone().or_else(|| {
                    let parts: Vec<&str> = class.branch.0.split('/').collect();
                    if parts.len() > 1 {
                        Some(parts[0].to_string())
                    } else {
                        Some("origin".to_string())
                    }
                })
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
}
