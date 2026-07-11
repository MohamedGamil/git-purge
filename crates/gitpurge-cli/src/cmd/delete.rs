//! Subcommand handlers for `delete` and `archive` (CLI Spec §8.4, §8.5).

use gitpurge_core::{
    action::ArchiveStrategy,
    model::{ActionKind, BranchName, ExecMode, MergeState, RepoId},
    Engine, GitPurgeError, Result,
};
use serde_json::json;

#[allow(clippy::too_many_arguments)]
pub fn handle_delete(
    engine: &Engine,
    repo_id: &RepoId,
    execute: bool,
    no_backup: bool,
    yes_global: bool,
    force_unmerged: bool,
    include_unmerged: bool,
    unmerged_only: bool,
    _continue_on_error: bool, // Note: engine.execute deletes all planned branches and reports successes/failures.
    flags: &crate::cli::SelectionFlags,
    json_output: bool,
) -> Result<()> {
    // 1. Plan the delete
    let mut filter = crate::cmd::scan::make_action_filter(flags, Some(ActionKind::Delete));

    // Adjust filter based on specific delete flags
    if unmerged_only {
        filter.merged_only = false;
        filter.include_unmerged = true;
    } else if include_unmerged {
        filter.include_unmerged = true;
    }

    let mut plan = engine.plan(repo_id, &filter)?;

    // If unmerged_only, keep only unmerged branches in the plan
    if unmerged_only {
        plan.actions
            .retain(|action| action.classification.merge_state == MergeState::Unmerged);
    }

    if plan.actions.is_empty() {
        if json_output {
            println!(
                "{}",
                json!({
                    "schema_version": "1",
                    "command": "delete",
                    "ok": true,
                    "dry_run": !execute,
                    "repo": repo_id.0,
                    "data": {
                        "success_count": 0,
                        "failure_count": 0,
                        "skipped_count": 0,
                        "results": []
                    },
                    "warnings": [],
                    "error": null
                })
            );
        } else {
            println!("No branches found to delete.");
        }
        return Ok(());
    }

    // Check if plan contains unmerged branches
    let has_unmerged = plan
        .actions
        .iter()
        .any(|action| action.classification.merge_state == MergeState::Unmerged);

    if !execute {
        // DRY-RUN mode: print plan
        if json_output {
            println!(
                "{}",
                json!({
                    "schema_version": "1",
                    "command": "delete",
                    "ok": true,
                    "dry_run": true,
                    "repo": repo_id.0,
                    "data": plan,
                    "warnings": [],
                    "error": null
                })
            );
        } else {
            println!(
                "Delete plan (dry-run) · repo={} · unmerged={}",
                repo_id.0, has_unmerged
            );
            let mut table = comfy_table::Table::new();
            table.set_header(vec!["ACTION", "REF", "REASON", "LAST COMMIT", "MERGED?"]);
            for action in &plan.actions {
                let merged_str = match action.classification.merge_state {
                    MergeState::Merged => "yes",
                    MergeState::Unmerged => "no",
                    MergeState::Unknown => "unknown",
                };
                let last_commit = format!(
                    "{} by {}",
                    crate::cmd::scan::format_age(action.classification.age),
                    action.classification.tip.author.name
                );
                table.add_row(vec![
                    "DELETE",
                    action.branch.0.as_str(),
                    action.reason.as_str(),
                    last_commit.as_str(),
                    merged_str,
                ]);
            }
            println!("{}", table);
            println!(
                "{} branches selected for deletion. Run with --execute to apply changes.",
                plan.actions.len()
            );
        }
        return Ok(());
    }

    // 2. Confirmation checks
    if has_unmerged {
        let prompt = format!(
            "WARNING: You are about to delete unmerged branches in repo '{}'. This is destructive and commits may be lost.",
            repo_id.0
        );
        if !crate::confirm::confirm_strong(&repo_id.0, &prompt, force_unmerged) {
            return Err(GitPurgeError::SafetyViolation(
                "Deletion of unmerged branches was not confirmed.".to_string(),
            ));
        }
    } else {
        let prompt = format!(
            "Are you sure you want to delete {} branches?",
            plan.actions.len()
        );
        if !crate::confirm::confirm_standard(&prompt, yes_global) {
            return Err(GitPurgeError::SafetyViolation(
                "Deletion was not confirmed.".to_string(),
            ));
        }
    }

    // 3. Execute deletion
    let report = engine.execute(&plan, ExecMode::Execute, no_backup)?;

    if json_output {
        println!(
            "{}",
            json!({
                "schema_version": "1",
                "command": "delete",
                "ok": true,
                "dry_run": false,
                "repo": repo_id.0,
                "data": report,
                "warnings": [],
                "error": null
            })
        );
    } else {
        println!("Execution completed successfully.");
        if let Some(ref snapshot_id) = report.snapshot {
            println!("Backup snapshot created: {}", snapshot_id.0);
        }
        println!(
            "Results: {} deleted · {} failed · {} skipped",
            report.success_count, report.failure_count, report.skipped_count
        );

        if report.failure_count > 0 {
            println!("\nFailures:");
            for result in &report.results {
                if let gitpurge_core::model::ActionResult::Failed { action, error } = result {
                    println!("  - {}: {}", action.branch.0, error);
                }
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn handle_archive(
    engine: &Engine,
    repo_id: &RepoId,
    execute: bool,
    target: &str,
    strategy: &crate::cli::MergeStrategyArg,
    push: bool,
    yes_global: bool,
    flags: &crate::cli::SelectionFlags,
    json_output: bool,
) -> Result<()> {
    // 1. Plan/scan for stale unmerged branches
    let scan_opts = crate::cmd::scan::make_scan_options(flags);
    let mut scan_result = engine.scan(repo_id, scan_opts)?;

    // Retain unmerged stale branches
    scan_result.classifications.retain(|c| {
        c.merge_state == MergeState::Unmerged && c.activity == gitpurge_core::model::Activity::Stale
    });

    let branches_to_archive: Vec<BranchName> = scan_result
        .classifications
        .iter()
        .map(|c| c.branch.clone())
        .collect();

    if branches_to_archive.is_empty() {
        if json_output {
            println!(
                "{}",
                json!({
                    "schema_version": "1",
                    "command": "archive",
                    "ok": true,
                    "dry_run": !execute,
                    "repo": repo_id.0,
                    "data": {
                        "archived_count": 0
                    },
                    "warnings": [],
                    "error": null
                })
            );
        } else {
            println!("No unmerged stale branches found to archive.");
        }
        return Ok(());
    }

    if !execute {
        if json_output {
            println!(
                "{}",
                json!({
                    "schema_version": "1",
                    "command": "archive",
                    "ok": true,
                    "dry_run": true,
                    "repo": repo_id.0,
                    "data": {
                        "target_branch": target,
                        "strategy": format!("{:?}", strategy),
                        "branches": branches_to_archive
                    },
                    "warnings": [],
                    "error": null
                })
            );
        } else {
            println!(
                "Archive plan (dry-run) · repo={} · target={} · strategy={:?}",
                repo_id.0, target, strategy
            );
            for b in &branches_to_archive {
                println!("  - {}", b.0);
            }
            println!(
                "{} branches selected for archiving. Run with --execute to apply changes.",
                branches_to_archive.len()
            );
        }
        return Ok(());
    }

    // Confirmation check
    let prompt = format!(
        "Are you sure you want to merge {} branches into '{}'?",
        branches_to_archive.len(),
        target
    );
    if !crate::confirm::confirm_standard(&prompt, yes_global) {
        return Err(GitPurgeError::SafetyViolation(
            "Archiving was not confirmed.".to_string(),
        ));
    }

    // 2. Perform archiving
    let core_strategy = match strategy {
        crate::cli::MergeStrategyArg::Ours => ArchiveStrategy::Ours,
        crate::cli::MergeStrategyArg::Theirs => ArchiveStrategy::Theirs,
    };

    engine.archive(repo_id, &branches_to_archive, target, core_strategy, push)?;

    if json_output {
        println!(
            "{}",
            json!({
                "schema_version": "1",
                "command": "archive",
                "ok": true,
                "dry_run": false,
                "repo": repo_id.0,
                "data": {
                    "archived_count": branches_to_archive.len(),
                    "target_branch": target,
                    "strategy": format!("{:?}", strategy)
                },
                "warnings": [],
                "error": null
            })
        );
    } else {
        println!(
            "Successfully archived {} branches into '{}'.",
            branches_to_archive.len(),
            target
        );
    }

    Ok(())
}
