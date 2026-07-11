//! Classification pipeline (docs/04 §2 — scan subsystem).

use crate::clock::Clock;
use crate::error::Result;
use crate::git::GitBackend;
use crate::model::{
    Activity, Branch, BranchName, Classification, MergeState, Protection, Recommendation, RefBasis,
    Repository, ScanResult, TrackingFacet,
};
use crate::policy::PolicyEngine;
pub mod filter;

pub use filter::filter_and_sort_classifications;
use rayon::prelude::*;

/// The scanner orchestrates reference reading and policy evaluation.
pub struct Scanner;

impl Scanner {
    /// Classify all branches of a repository using the provided policy and clock.
    pub fn classify(
        git: &dyn GitBackend,
        repo: &Repository,
        policy_engine: &PolicyEngine,
        clock: &dyn Clock,
        branches: Vec<Branch>,
    ) -> Result<ScanResult> {
        let total_branches = branches.len();

        // Resolve the default branch
        let default_branch_name = Self::resolve_default_branch(&branches, policy_engine);

        let default_branch_tip_oid = default_branch_name.as_ref().and_then(|def_name| {
            branches
                .iter()
                .find(|b| b.name.0 == def_name.0)
                .map(|b| b.tip.oid.clone())
        });

        let classifications_res: Result<Vec<Option<Classification>>> = branches
            .into_par_iter()
            .map(|branch| {
                // Check exclusion
                if policy_engine.is_excluded(&branch.name.0) {
                    return Ok(None);
                }

                // Calculate age (now - tip.commit_date)
                let age_duration = clock.now() - branch.tip.commit_date;
                let age =
                    std::time::Duration::from_secs(age_duration.whole_seconds().max(0) as u64);

                // Activity
                let activity = if age >= policy_engine.age_duration {
                    Activity::Stale
                } else {
                    Activity::Active
                };

                // Protection
                let is_default = Some(&branch.name) == default_branch_name.as_ref();
                let protection =
                    policy_engine
                        .protection
                        .evaluate(&branch.name.0, is_default, branch.is_head);

                // Naming
                let naming = policy_engine.naming.evaluate(&branch.name.0);

                // Merge state
                let merge_state = if is_default {
                    // Default branch is not merged into itself/others for cleanup purposes
                    MergeState::Unmerged
                } else if let Some(ref default_oid) = default_branch_tip_oid {
                    match git.is_ancestor(repo, &branch.tip.oid, default_oid) {
                        Ok(true) => MergeState::Merged,
                        Ok(false) => MergeState::Unmerged,
                        Err(e) => return Err(e),
                    }
                } else {
                    MergeState::Unknown
                };

                // Tracking facet
                let tracking = if let Some(u) = &branch.upstream {
                    TrackingFacet {
                        ahead: u.ahead,
                        behind: u.behind,
                        upstream_gone: false,
                        compared_against: RefBasis::Upstream,
                    }
                } else {
                    TrackingFacet {
                        ahead: 0,
                        behind: 0,
                        upstream_gone: false,
                        compared_against: RefBasis::DefaultBranch,
                    }
                };

                // Recommendation
                let recommendation = if !matches!(protection, Protection::Unprotected) {
                    Recommendation::KeepProtected
                } else if merge_state == MergeState::Merged {
                    Recommendation::DeleteMerged
                } else if activity == Activity::Stale {
                    Recommendation::ArchiveStale
                } else {
                    Recommendation::ReviewUnmerged
                };

                Ok(Some(Classification {
                    branch: branch.name.clone(),
                    scope: branch.scope,
                    merge_state,
                    activity,
                    age,
                    protection,
                    naming,
                    tracking,
                    tip: branch.tip.clone(),
                    recommendation,
                }))
            })
            .collect();

        let classifications_opt = classifications_res?;
        let mut classifications = Vec::new();
        let mut excluded_count = 0;
        for c in classifications_opt {
            if let Some(class) = c {
                classifications.push(class);
            } else {
                excluded_count += 1;
            }
        }

        Ok(ScanResult {
            repo: repo.id.clone(),
            classifications,
            total_branches,
            excluded_count,
        })
    }

    /// Resolve the default branch name from the branch list and the policy.
    pub fn resolve_default_branch(
        branches: &[Branch],
        policy_engine: &PolicyEngine,
    ) -> Option<BranchName> {
        let policy = &policy_engine.policy.default_branch;

        // 1. Explicit config
        if let Some(explicit) = &policy.explicit {
            if branches.iter().any(|b| b.name.0 == explicit.0) {
                return Some(explicit.clone());
            }
        }

        // 2. Remote HEAD / active HEAD
        if let Some(head_branch) = branches.iter().find(|b| b.is_head) {
            return Some(head_branch.name.clone());
        }

        // 3. Candidates
        for candidate in &policy.candidates {
            if branches.iter().any(|b| b.name.0 == candidate.0) {
                return Some(candidate.clone());
            }
        }

        // Fallback: first branch
        branches.first().map(|b| b.name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::FakeClock;
    use crate::git::GixBackend;
    use crate::model::Policy;
    use crate::testkit;

    #[test]
    fn test_scanner_classification() {
        let repo_fixture = testkit::merged_repo();
        let backend = GixBackend;

        let repo_model = Repository {
            id: crate::model::RepoId("test-scan".to_string()),
            display_name: "test-scan".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };

        let policy = Policy::default();
        let engine = PolicyEngine::new(policy).unwrap();

        // Pinned clock to ensure deterministic age calculation
        // Main branch commit was at 2026-07-02T12:00:00Z.
        // We set current clock to 2026-07-05T12:00:00Z.
        let now = time::macros::datetime!(2026-07-05 12:00:00 UTC);
        let clock = FakeClock::new(now);

        let branches = backend.list_branches(&repo_model, None).unwrap();
        let scan_result =
            Scanner::classify(&backend, &repo_model, &engine, &clock, branches).unwrap();

        assert_eq!(scan_result.total_branches, 3);

        // Verify merged-branch is classified as Merged
        let merged_class = scan_result
            .classifications
            .iter()
            .find(|c| c.branch.0 == "merged-branch")
            .unwrap();
        assert_eq!(merged_class.merge_state, MergeState::Merged);
        assert_eq!(merged_class.recommendation, Recommendation::DeleteMerged);

        // Verify unmerged-branch is classified as Unmerged
        let unmerged_class = scan_result
            .classifications
            .iter()
            .find(|c| c.branch.0 == "unmerged-branch")
            .unwrap();
        assert_eq!(unmerged_class.merge_state, MergeState::Unmerged);
        assert_eq!(
            unmerged_class.recommendation,
            Recommendation::ReviewUnmerged
        );

        // Verify main is protected because it's head / default branch
        let main_class = scan_result
            .classifications
            .iter()
            .find(|c| c.branch.0 == "main")
            .unwrap();
        assert_ne!(main_class.protection, Protection::Unprotected);
        assert_eq!(main_class.recommendation, Recommendation::KeepProtected);
    }
}
