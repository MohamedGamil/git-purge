//! Guarded execution wrapper (P2-T6, CONVENTIONS §7.5, docs/08 §7.3).

use crate::error::Result;
use crate::model::{Repository, SnapshotId, RestoreSpec, BranchName, ActionResult, Action, ActionKind};
use crate::config::Config;
use crate::git::GitBackend;
use crate::history::HistoryStore;

/// Execute a list of branch deletions with a pre-op backup snapshot and verification.
/// If any delete fails, calls `offer_restore` callback to prompt for auto-restore (SAFE-05).
#[allow(clippy::too_many_arguments)]
pub fn execute_deletions_with_guard<F>(
    config: &Config,
    git_backend: &dyn GitBackend,
    history_store: &dyn HistoryStore,
    repo: &Repository,
    classifications: &[crate::model::Classification],
    branches_to_delete: &[BranchName],
    mut delete_fn: impl FnMut(&BranchName) -> Result<()>,
    mut offer_restore: F,
) -> Result<Vec<ActionResult>>
where
    F: FnMut(&BranchName, &SnapshotId) -> bool,
{
    if branches_to_delete.is_empty() {
        return Ok(Vec::new());
    }

    // 1. Create a pre-op snapshot (SAFE-04)
    let backup_opts = crate::model::BackupOptions {
        trigger: Some(crate::model::SnapshotTrigger::PreDelete),
        verify: true,
        only_branches: branches_to_delete.to_vec(),
    };

    let mut snapshot = crate::backup::create_snapshot(
        config,
        git_backend,
        repo,
        classifications,
        &backup_opts,
    )?;

    // 2. SAFE-04: Verify the snapshot before taking any destructive action
    let verify_report = crate::backup::verify_snapshot(config, &repo.id, &snapshot.id, false)?;
    if !verify_report.ok {
        return Err(crate::GitPurgeError::Snapshot(format!(
            "Pre-op backup snapshot '{}' failed verification. Aborting operation for safety.",
            snapshot.id.0
        )));
    }

    snapshot.verified = true;

    // Save the snapshot in history
    history_store.save_snapshot(&snapshot)?;

    let mut results = Vec::new();

    // 3. Execute the delete operation for each branch
    for branch_name in branches_to_delete {
        let class = classifications.iter()
            .find(|c| c.branch == *branch_name)
            .cloned()
            .unwrap_or_else(|| {
                // fallback if not found in classifications
                crate::model::Classification {
                    branch: branch_name.clone(),
                    scope: crate::model::BranchScope::Local,
                    merge_state: crate::model::MergeState::Unknown,
                    activity: crate::model::Activity::Active,
                    age: std::time::Duration::from_secs(0),
                    protection: crate::model::Protection::Unprotected,
                    naming: crate::model::NamingVerdict::Standard,
                    tracking: crate::model::TrackingFacet {
                        ahead: 0,
                        behind: 0,
                        upstream_gone: false,
                        compared_against: crate::model::RefBasis::DefaultBranch,
                    },
                    tip: crate::model::Commit {
                        oid: crate::model::Oid("0000000000000000000000000000000000000000".to_string()),
                        short: "0000000".to_string(),
                        author: crate::model::Signature {
                            name: "System".to_string(),
                            email: "system@gitpurge".to_string(),
                            when: time::OffsetDateTime::now_utc(),
                        },
                        committer: crate::model::Signature {
                            name: "System".to_string(),
                            email: "system@gitpurge".to_string(),
                            when: time::OffsetDateTime::now_utc(),
                        },
                        author_date: time::OffsetDateTime::now_utc(),
                        commit_date: time::OffsetDateTime::now_utc(),
                        subject: "Initial".to_string(),
                        parents: Vec::new(),
                    },
                    recommendation: crate::model::Recommendation::NoAction,
                }
            });

        let action = Action {
            kind: ActionKind::Delete,
            branch: branch_name.clone(),
            scope: class.scope,
            remote: if class.scope == crate::model::BranchScope::Remote {
                Some("origin".to_string())
            } else {
                None
            },
            classification: class,
            reason: "Safe branch deletion".to_string(),
        };

        match delete_fn(branch_name) {
            Ok(()) => {
                results.push(ActionResult::Success { action });
            }
            Err(e) => {
                let error_msg = e.to_string();
                results.push(ActionResult::Failed {
                    action,
                    error: error_msg.clone(),
                });

                // SAFE-05: Offer restore
                if offer_restore(branch_name, &snapshot.id) {
                    let restore_spec = RestoreSpec {
                        branch: branch_name.clone(),
                        as_tag: false,
                        target_name: None,
                        force: true, // force since we are restoring from failed delete
                    };
                    let _ = crate::backup::restore_snapshot(config, repo, &snapshot.id, &restore_spec);
                }
            }
        }
    }

    Ok(results)
}
