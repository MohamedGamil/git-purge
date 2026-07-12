//! Subcommand handlers for `backup` and `restore` (CLI Spec §8.6, §8.7).

use gitpurge_core::{
    model::{
        BackupOptions, BranchName, ExecMode, RepoId, RestoreSpec, RetentionPolicy, SnapshotId,
        SnapshotTrigger,
    },
    Engine, GitPurgeError, Result,
};
use serde_json::json;

pub fn handle_backup(
    engine: &Engine,
    repo_id: &RepoId,
    action: &crate::cli::BackupAction,
    execute: bool,
    json_output: bool,
) -> Result<()> {
    match action {
        crate::cli::BackupAction::Create { note, refs } => {
            let only_branches = refs
                .as_ref()
                .map(|s| {
                    s.split(',')
                        .map(|b| BranchName(b.trim().to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let opts = BackupOptions {
                trigger: Some(SnapshotTrigger::Manual),
                verify: true,
                only_branches,
            };

            let snapshot = engine.backup_create(repo_id, opts)?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "backup create",
                        "ok": true,
                        "dry_run": false,
                        "repo": repo_id.0,
                        "data": snapshot,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                println!("Backup snapshot created successfully: {}", snapshot.id.0);
                if let Some(ref note_str) = note {
                    println!("Note: {}", note_str);
                }
                println!("Captured {} references.", snapshot.refs.len());
            }
        }
        crate::cli::BackupAction::List => {
            let snapshots = engine.list_snapshots(repo_id)?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "backup list",
                        "ok": true,
                        "dry_run": false,
                        "repo": repo_id.0,
                        "data": snapshots,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                if snapshots.is_empty() {
                    println!("No snapshots found for repository '{}'.", repo_id.0);
                    return Ok(());
                }

                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "SNAPSHOT ID",
                    "CREATED AT",
                    "TRIGGER",
                    "REFS COUNT",
                    "VERIFIED",
                ]);

                for s in snapshots {
                    let trigger_str = format!("{:?}", s.trigger);
                    let verified_str = if s.verified { "yes" } else { "no" };
                    table.add_row(vec![
                        s.id.0.as_str(),
                        s.created_at.to_string().as_str(),
                        trigger_str.as_str(),
                        s.refs.len().to_string().as_str(),
                        verified_str,
                    ]);
                }
                println!("{}", table);
            }
        }
        crate::cli::BackupAction::Show { snapshot_id } => {
            let snap_id = SnapshotId(snapshot_id.clone());
            let snapshot = engine.get_snapshot(&snap_id)?.ok_or_else(|| {
                GitPurgeError::Snapshot(format!("Snapshot not found: {}", snapshot_id))
            })?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "backup show",
                        "ok": true,
                        "dry_run": false,
                        "repo": repo_id.0,
                        "data": snapshot,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                println!("Snapshot Details");
                println!("ID:          {}", snapshot.id.0);
                println!("Created At:  {}", snapshot.created_at);
                println!("Trigger:     {:?}", snapshot.trigger);
                println!(
                    "Verified:    {}",
                    if snapshot.verified { "yes" } else { "no" }
                );
                println!("Manifest:    {}", snapshot.manifest_path.to_string_lossy());
                println!("\nCaptured References:");
                for r in &snapshot.refs {
                    println!("  - {} -> {}", r.branch.0, r.backup_ref);
                }
            }
        }
        crate::cli::BackupAction::Verify { snapshot_id } => {
            let snap_id = SnapshotId(snapshot_id.clone());
            let report = engine.backup_verify(repo_id, &snap_id)?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "backup verify",
                        "ok": report.ok,
                        "dry_run": false,
                        "repo": repo_id.0,
                        "data": report,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                if report.ok {
                    println!("Integrity check PASSED for snapshot '{}'.", snapshot_id);
                } else {
                    println!("Integrity check FAILED for snapshot '{}'.", snapshot_id);
                    println!("\nProblems found:");
                    for p in report.problems {
                        println!("  - {:?}", p);
                    }
                }
            }
        }
        crate::cli::BackupAction::Prune { keep, older_than } => {
            let keep_last = keep.map(|k| k as usize);
            let keep_within = if let Some(ref o) = older_than {
                match gitpurge_core::model::AgeThreshold::parse(o.clone()) {
                    Ok(threshold) => Some(threshold.duration),
                    Err(e) => {
                        return Err(GitPurgeError::Config(format!(
                            "Invalid duration format for --older-than: {}",
                            e
                        )));
                    }
                }
            } else {
                None
            };

            let policy = RetentionPolicy {
                keep_last,
                keep_within,
                min_keep: 1, // keep at least 1 backup for safety (SAFE-04)
                keep_triggers: vec![SnapshotTrigger::Manual],
            };

            let mode = if execute {
                ExecMode::Execute
            } else {
                ExecMode::DryRun
            };
            let report = engine.backup_prune(repo_id, &policy, mode)?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "backup prune",
                        "ok": true,
                        "dry_run": !execute,
                        "repo": repo_id.0,
                        "data": report,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                if !execute {
                    println!(
                        "[DRY-RUN] Would prune {} snapshots.",
                        report.pruned_snapshots.len()
                    );
                    for s in &report.pruned_snapshots {
                        println!("  - {}", s.0);
                    }
                    println!(
                        "Space that would be reclaimed: {} bytes",
                        report.space_reclaimed_bytes
                    );
                    println!("Run with --execute to apply changes.");
                } else {
                    println!(
                        "Successfully pruned {} snapshots.",
                        report.pruned_snapshots.len()
                    );
                    println!("Reclaimed space: {} bytes", report.space_reclaimed_bytes);
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn handle_restore(
    engine: &Engine,
    repo_id: &RepoId,
    snapshot_id: &str,
    ref_or_glob: &str,
    as_tag: bool,
    as_name: &Option<String>,
    force: bool,
    execute: bool,
    json_output: bool,
) -> Result<()> {
    // 1. Resolve snapshot_id (latest or exact)
    let snap_id = if snapshot_id.to_lowercase() == "latest" {
        let list = engine.list_snapshots(repo_id)?;
        let latest = list.first().ok_or_else(|| {
            GitPurgeError::Snapshot(
                "No snapshots exist for repository to restore from.".to_string(),
            )
        })?;
        latest.id.clone()
    } else {
        SnapshotId(snapshot_id.to_string())
    };

    let spec = RestoreSpec {
        branch: BranchName(ref_or_glob.to_string()),
        as_tag,
        target_name: as_name.clone(),
        force,
        original_ref: None,
    };

    if !execute {
        if json_output {
            println!(
                "{}",
                json!({
                    "schema_version": "1",
                    "command": "restore",
                    "ok": true,
                    "dry_run": true,
                    "repo": repo_id.0,
                    "data": {
                        "snapshot": snap_id.0,
                        "spec": spec
                    },
                    "warnings": [],
                    "error": null
                })
            );
        } else {
            println!(
                "[DRY-RUN] Would restore branch '{}' from snapshot '{}'.",
                ref_or_glob, snap_id.0
            );
            if as_tag {
                println!("  - Restoring as a TAG");
            }
            if let Some(ref target) = as_name {
                println!("  - Target name: {}", target);
            }
            println!("Run with --execute to apply changes.");
        }
        return Ok(());
    }

    // 2. Perform restoration
    let outcome = engine.restore(&snap_id, spec)?;

    if json_output {
        println!(
            "{}",
            json!({
                "schema_version": "1",
                "command": "restore",
                "ok": true,
                "dry_run": false,
                "repo": repo_id.0,
                "data": outcome,
                "warnings": [],
                "error": null
            })
        );
    } else {
        println!(
            "Successfully restored '{}' to '{}' from snapshot '{}'.",
            outcome.branch.0, outcome.created_ref, outcome.snapshot.0
        );
    }

    Ok(())
}
