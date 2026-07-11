//! Subcommand handlers for `scan` and `plan` commands (CLI Spec §8.2, §8.3).

use comfy_table::Table;
use gitpurge_core::{
    model::{ActionFilter, ActionKind, BranchScope, GlobPattern, MergeState, RepoId, ScanOptions},
    Engine, Result,
};
use serde_json::json;

pub fn make_scan_options(flags: &crate::cli::SelectionFlags) -> ScanOptions {
    let excludes = flags
        .exclude
        .as_ref()
        .map(|s| {
            s.split(',')
                .map(|g| GlobPattern(g.trim().to_string()))
                .collect()
        })
        .unwrap_or_default();

    let scope = match (flags.local, flags.remote) {
        (true, false) => Some(BranchScope::Local),
        (false, true) => Some(BranchScope::Remote),
        _ => None,
    };

    ScanOptions {
        scope,
        age_override: flags.age.clone(),
        excludes,
        include_all: flags.standard && flags.non_standard,
    }
}

pub fn make_action_filter(
    flags: &crate::cli::SelectionFlags,
    kind: Option<ActionKind>,
) -> ActionFilter {
    let exclude_globs = flags
        .exclude
        .as_ref()
        .map(|s| {
            s.split(',')
                .map(|g| GlobPattern(g.trim().to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Map protected CSV to include_globs if needed, but in our case protected refs are exclusions.
    // In ActionFilter, exclude_globs maps to excludes.
    ActionFilter {
        kind,
        merged_only: flags.merged && !flags.unmerged,
        include_unmerged: flags.unmerged,
        include_globs: Vec::new(),
        exclude_globs,
        age_override: flags.age.clone(),
        specific_branches: Vec::new(),
    }
}

pub fn format_age(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    let months = days / 30;
    let years = days / 365;

    if years > 0 {
        let rem_months = months % 12;
        if rem_months > 0 {
            format!("{}y {}m", years, rem_months)
        } else {
            format!("{}y", years)
        }
    } else if months > 0 {
        let rem_days = days % 30;
        if rem_days > 0 {
            format!("{}m {}d", months, rem_days)
        } else {
            format!("{}m", months)
        }
    } else if days > 0 {
        format!("{}d", days)
    } else if hours > 0 {
        format!("{}h", hours)
    } else {
        "just now".to_string()
    }
}

pub fn handle_scan(
    engine: &Engine,
    repo_id: &RepoId,
    no_refresh: bool,
    flags: &crate::cli::SelectionFlags,
    json_output: bool,
) -> Result<()> {
    if !no_refresh {
        // Refresh from remote
        if let Some(repo) = engine.get_repo(repo_id)? {
            // Fetch if has remote
            if repo.remote_url.is_some() {
                if let Err(e) = engine.fetch(repo_id) {
                    eprintln!(
                        "Warning: Failed to fetch from remote: {}. Operating on local cache.",
                        e
                    );
                }
            }
        }
    }

    let scan_opts = make_scan_options(flags);
    let mut scan_result = engine.scan(repo_id, scan_opts)?;

    // Apply standard / non-standard / protection filters manually to match SelectionFlags
    scan_result.classifications.retain(|c| {
        let is_standard = matches!(
            c.naming,
            gitpurge_core::model::NamingVerdict::Standard
                | gitpurge_core::model::NamingVerdict::Exempt { .. }
        );
        if flags.standard && !is_standard {
            return false;
        }
        if flags.non_standard && is_standard {
            return false;
        }
        if flags.merged && c.merge_state != MergeState::Merged {
            return false;
        }
        if flags.unmerged && c.merge_state == MergeState::Merged {
            return false;
        }
        true
    });

    // Apply sorting
    if let Some(ref sort_key) = flags.sort {
        match sort_key.to_lowercase().as_str() {
            "name" => scan_result
                .classifications
                .sort_by(|a, b| a.branch.0.cmp(&b.branch.0)),
            "age" => scan_result
                .classifications
                .sort_by_key(|a| a.age),
            "author" => scan_result
                .classifications
                .sort_by(|a, b| a.tip.author.name.cmp(&b.tip.author.name)),
            "ahead" => scan_result
                .classifications
                .sort_by_key(|b| std::cmp::Reverse(b.tracking.ahead)),
            "behind" => scan_result
                .classifications
                .sort_by_key(|b| std::cmp::Reverse(b.tracking.behind)),
            _ => {}
        }
    } else {
        // Default sort by age
        scan_result
            .classifications
            .sort_by_key(|b| std::cmp::Reverse(b.age));
    }

    // Apply limit
    if let Some(limit) = flags.limit {
        scan_result.classifications.truncate(limit as usize);
    }

    if json_output {
        println!(
            "{}",
            json!({
                "schema_version": "1",
                "command": "scan",
                "ok": true,
                "dry_run": false,
                "repo": repo_id.0,
                "data": scan_result,
                "warnings": [],
                "error": null
            })
        );
    } else {
        // Print header summary
        let active_count = scan_result
            .classifications
            .iter()
            .filter(|c| c.activity == gitpurge_core::model::Activity::Active)
            .count();
        let stale_count = scan_result
            .classifications
            .iter()
            .filter(|c| c.activity == gitpurge_core::model::Activity::Stale)
            .count();
        let merged_count = scan_result
            .classifications
            .iter()
            .filter(|c| c.merge_state == MergeState::Merged)
            .count();
        let unmerged_count = scan_result
            .classifications
            .iter()
            .filter(|c| c.merge_state == MergeState::Unmerged)
            .count();
        let non_std_count = scan_result
            .classifications
            .iter()
            .filter(|c| {
                !matches!(
                    c.naming,
                    gitpurge_core::model::NamingVerdict::Standard
                        | gitpurge_core::model::NamingVerdict::Exempt { .. }
                )
            })
            .count();

        println!("Scanning {}...", repo_id.0);
        println!(
            "Metrics  total {} · active {} · stale {} · merged {} · unmerged {} · non-standard {}",
            scan_result.total_branches,
            active_count,
            stale_count,
            merged_count,
            unmerged_count,
            non_std_count
        );

        if scan_result.classifications.is_empty() {
            println!("No branches found matching the filters.");
            return Ok(());
        }

        let mut table = Table::new();
        table.set_header(vec![
            "BRANCH",
            "AGE",
            "MERGED",
            "STD",
            "AHEAD/BEHIND",
            "AUTHOR",
        ]);

        for c in scan_result.classifications {
            let age_str = format_age(c.age);
            let merged_str = match c.merge_state {
                MergeState::Merged => "yes",
                MergeState::Unmerged => "no",
                MergeState::Unknown => "unknown",
            };
            let std_str = if matches!(
                c.naming,
                gitpurge_core::model::NamingVerdict::Standard
                    | gitpurge_core::model::NamingVerdict::Exempt { .. }
            ) {
                "yes"
            } else {
                "no"
            };
            let ahead_behind = format!("{}/{}", c.tracking.ahead, c.tracking.behind);
            table.add_row(vec![
                c.branch.0.as_str(),
                age_str.as_str(),
                merged_str,
                std_str,
                ahead_behind.as_str(),
                c.tip.author.name.as_str(),
            ]);
        }
        println!("{}", table);
    }

    Ok(())
}

pub fn handle_plan(
    engine: &Engine,
    repo_id: &RepoId,
    action_type: &crate::cli::ActionType,
    flags: &crate::cli::SelectionFlags,
    json_output: bool,
) -> Result<()> {
    let kind = match action_type {
        crate::cli::ActionType::Delete => ActionKind::Delete,
        crate::cli::ActionType::Archive => ActionKind::Archive,
    };

    let filter = make_action_filter(flags, Some(kind));
    let plan = engine.plan(repo_id, &filter)?;

    if json_output {
        println!(
            "{}",
            json!({
                "schema_version": "1",
                "command": "plan",
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
            "Plan (dry-run) · action={:?} · age≤{} · pre-op backup: WOULD create",
            action_type,
            flags.age.as_deref().unwrap_or("default")
        );

        if plan.actions.is_empty() {
            println!("No actions to execute.");
            return Ok(());
        }

        let mut table = Table::new();
        table.set_header(vec!["ACTION", "REF", "REASON", "LAST COMMIT", "MERGED?"]);

        for action in &plan.actions {
            let action_str = match action.kind {
                ActionKind::Delete => "DELETE",
                ActionKind::Archive => "ARCHIVE",
                ActionKind::Restore => "RESTORE",
            };
            let merged_str = match action.classification.merge_state {
                MergeState::Merged => "yes",
                MergeState::Unmerged => "no",
                MergeState::Unknown => "unknown",
            };
            let last_commit = format!(
                "{} by {}",
                format_age(action.classification.age),
                action.classification.tip.author.name
            );

            table.add_row(vec![
                action_str,
                action.branch.0.as_str(),
                action.reason.as_str(),
                last_commit.as_str(),
                merged_str,
            ]);
        }
        println!("{}", table);

        let delete_count = plan
            .actions
            .iter()
            .filter(|a| a.kind == ActionKind::Delete)
            .count();
        let archive_count = plan
            .actions
            .iter()
            .filter(|a| a.kind == ActionKind::Archive)
            .count();
        println!(
            "{} to delete, {} to archive · run with --execute to apply",
            delete_count, archive_count
        );
    }

    Ok(())
}
