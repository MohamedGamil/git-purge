//! Programmatic branch deletion orchestration (P10-T1).

use crate::config::Config;
use crate::error::Result;
use crate::git::GitBackend;
use crate::history::HistoryStore;
use crate::model::{Action, ActionResult, Repository};
use crate::progress::ProgressSink;

/// Execute a list of branch deletions with safety guards and pre-op backup.
#[allow(clippy::too_many_arguments)]
pub fn delete_branches(
    config: &Config,
    git: &dyn GitBackend,
    history: &dyn HistoryStore,
    repo: &Repository,
    actions: &[Action],
    no_backup: bool,
    progress: &dyn ProgressSink,
    action_name: &str,
) -> Result<Vec<ActionResult>> {
    if actions.is_empty() {
        return Ok(Vec::new());
    }

    let mut current_step = 0;
    let total_steps = actions.len();
    progress.set_total(total_steps as u64);

    crate::action::guard::execute_deletions_with_guard(
        config,
        git,
        history,
        repo,
        actions,
        no_backup,
        |action| {
            current_step += 1;
            let msg = format!(
                "{} ({}/{}) branch {}",
                action_name, current_step, total_steps, action.branch.0
            );
            progress.tick(Some(&msg));

            if action.scope == crate::model::BranchScope::Remote {
                let remote = action.remote.as_deref().unwrap_or("origin");
                git.delete_remote_branch(repo, remote, &action.branch)
            } else {
                git.delete_local_branch(repo, &action.branch)
            }
        },
        |_, _| true,
    )
}
