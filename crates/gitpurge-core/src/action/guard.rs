//! Guarded execution wrapper (P2-T6, CONVENTIONS §7.5, docs/08 §7.3).

use crate::config::Config;
use crate::error::Result;
use crate::git::GitBackend;
use crate::history::HistoryStore;
use crate::model::{Action, ActionResult, BranchName, Repository, RestoreSpec, SnapshotId};

/// Execute a list of branch deletions with a pre-op backup snapshot and verification.
/// If any delete fails, calls `offer_restore` callback to prompt for auto-restore (SAFE-05).
#[allow(clippy::too_many_arguments)]
pub fn execute_deletions_with_guard<D, F>(
    config: &Config,
    git_backend: &dyn GitBackend,
    history_store: &dyn HistoryStore,
    repo: &Repository,
    actions_to_delete: &[Action],
    no_backup: bool,
    delete_fn: D,
    mut offer_restore: F,
) -> Result<Vec<ActionResult>>
where
    D: Fn(&Action) -> Result<()> + Send + Sync,
    F: FnMut(&BranchName, &SnapshotId) -> bool,
{
    if actions_to_delete.is_empty() {
        return Ok(Vec::new());
    }

    let snapshot_id = if no_backup {
        None
    } else {
        let branches_to_backup: Vec<BranchName> =
            actions_to_delete.iter().map(|a| a.branch.clone()).collect();

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

    let results = std::sync::Mutex::new(Vec::new());
    let mut failures = Vec::new();

    enum DeleteEvent {
        Success(Action),
        Failed(Action, String),
    }

    // 3. Execute the delete operation for each action concurrently
    std::thread::scope(|s| {
        let (tx, rx) = std::sync::mpsc::channel();

        for action in actions_to_delete {
            let tx = tx.clone();
            let delete_fn = &delete_fn;
            s.spawn(move || {
                let scope_str = match action.scope {
                    crate::model::BranchScope::Local => "local",
                    crate::model::BranchScope::Remote => "remote",
                };

                match delete_fn(action) {
                    Ok(()) => {
                        crate::log_operation("DELETE", &action.branch.0, scope_str, "SUCCESS");
                        tx.send(DeleteEvent::Success(action.clone())).ok();
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        crate::log_operation(
                            "DELETE",
                            &action.branch.0,
                            scope_str,
                            &format!("FAILED: {}", error_msg),
                        );
                        tx.send(DeleteEvent::Failed(action.clone(), error_msg)).ok();
                    }
                }
            });
        }
        drop(tx);

        while let Ok(event) = rx.recv() {
            match event {
                DeleteEvent::Success(action) => {
                    results
                        .lock()
                        .unwrap()
                        .push(ActionResult::Success { action });
                }
                DeleteEvent::Failed(action, error_msg) => {
                    results.lock().unwrap().push(ActionResult::Failed {
                        action: action.clone(),
                        error: error_msg.clone(),
                    });
                    failures.push((action, error_msg));
                }
            }
        }
    });

    // Handle offer_restore sequentially on the main thread for all failures
    for (action, _error_msg) in failures {
        if let Some(ref snap_id) = snapshot_id {
            if offer_restore(&action.branch, snap_id) {
                let original_ref = if action.scope == crate::model::BranchScope::Remote {
                    let remote = action.remote.as_deref().unwrap_or("origin");
                    format!("refs/remotes/{}/{}", remote, action.branch.0)
                } else {
                    format!("refs/heads/{}", action.branch.0)
                };
                let restore_spec = RestoreSpec {
                    branch: action.branch.clone(),
                    as_tag: false,
                    target_name: None,
                    force: true, // force since we are restoring from failed delete
                    original_ref: Some(original_ref),
                };
                let _ = crate::backup::restore_snapshot(config, repo, snap_id, &restore_spec);
            }
        }
    }

    let mut final_results = results.into_inner().unwrap();
    // Maintain the original order of actions for the results vector
    final_results.sort_by_key(|r| {
        let action = match r {
            ActionResult::Success { action } => action,
            ActionResult::Failed { action, .. } => action,
            ActionResult::Skipped { action } => action,
        };
        actions_to_delete
            .iter()
            .position(|a| a.branch == action.branch)
            .unwrap_or(0)
    });

    Ok(final_results)
}
