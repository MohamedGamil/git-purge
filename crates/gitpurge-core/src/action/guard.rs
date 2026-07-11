//! Guarded execution wrapper (P2-T6, CONVENTIONS §7.5, docs/08 §7.3).

use crate::config::Config;
use crate::error::Result;
use crate::git::GitBackend;
use crate::history::HistoryStore;
use crate::model::{
    Action, ActionResult, BranchName, Repository, RestoreSpec, SnapshotId,
};

/// Execute a list of branch deletions with a pre-op backup snapshot and verification.
/// If any delete fails, calls `offer_restore` callback to prompt for auto-restore (SAFE-05).
#[allow(clippy::too_many_arguments)]
pub fn execute_deletions_with_guard<F>(
    config: &Config,
    git_backend: &dyn GitBackend,
    history_store: &dyn HistoryStore,
    repo: &Repository,
    actions_to_delete: &[Action],
    no_backup: bool,
    mut delete_fn: impl FnMut(&Action) -> Result<()>,
    mut offer_restore: F,
) -> Result<Vec<ActionResult>>
where
    F: FnMut(&BranchName, &SnapshotId) -> bool,
{
    if actions_to_delete.is_empty() {
        return Ok(Vec::new());
    }

    let snapshot_id = if no_backup {
        None
    } else {
        let branches_to_backup: Vec<BranchName> = actions_to_delete
            .iter()
            .map(|a| a.branch.clone())
            .collect();

        let classifications: Vec<_> = actions_to_delete
            .iter()
            .map(|a| a.classification.clone())
            .collect();

        // 1. Create a pre-op snapshot (SAFE-04)
        let backup_opts = crate::model::BackupOptions {
            trigger: Some(crate::model::SnapshotTrigger::PreDelete),
            verify: true,
            only_branches: branches_to_backup,
        };

        let mut snapshot = crate::backup::create_snapshot(
            config,
            git_backend,
            repo,
            &classifications,
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
        Some(snapshot.id)
    };

    let mut results = Vec::new();

    // 3. Execute the delete operation for each action
    for action in actions_to_delete {
        let scope_str = match action.scope {
            crate::model::BranchScope::Local => "local",
            crate::model::BranchScope::Remote => "remote",
        };

        match delete_fn(action) {
            Ok(()) => {
                results.push(ActionResult::Success { action: action.clone() });
                crate::log_operation("DELETE", &action.branch.0, scope_str, "SUCCESS");
            }
            Err(e) => {
                let error_msg = e.to_string();
                results.push(ActionResult::Failed {
                    action: action.clone(),
                    error: error_msg.clone(),
                });
                crate::log_operation("DELETE", &action.branch.0, scope_str, &format!("FAILED: {}", error_msg));

                // SAFE-05: Offer restore
                if let Some(ref snap_id) = snapshot_id {
                    if offer_restore(&action.branch, snap_id) {
                        let restore_spec = RestoreSpec {
                            branch: action.branch.clone(),
                            as_tag: false,
                            target_name: None,
                            force: true, // force since we are restoring from failed delete
                        };
                        let _ =
                            crate::backup::restore_snapshot(config, repo, snap_id, &restore_spec);
                    }
                }
            }
        }
    }

    Ok(results)
}
