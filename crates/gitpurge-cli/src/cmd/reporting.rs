#![allow(deprecated)]
// reporting.rs — CLI reporting command handlers (Phase P5)

use gitpurge_core::{
    model::RepoId,
    report::{ReportFormat, ReportType},
    Engine, Result,
};

/// Handle the `report` CLI command.
pub fn handle_report(
    engine: &Engine,
    repo_id: &RepoId,
    report_type: crate::cli::ReportType,
    format_arg: crate::cli::ReportFormatArg,
    out: Option<String>,
    _baseline: Option<String>,
    json_output: bool,
) -> Result<()> {
    let core_format = match format_arg {
        crate::cli::ReportFormatArg::Md => ReportFormat::Markdown,
        crate::cli::ReportFormatArg::Json => ReportFormat::Json,
        crate::cli::ReportFormatArg::Html => ReportFormat::Html,
    };

    let report_types = match report_type {
        crate::cli::ReportType::Audit => vec![ReportType::Audit],
        crate::cli::ReportType::Trend => vec![ReportType::Trend],
        crate::cli::ReportType::Both => vec![ReportType::Audit, ReportType::Trend],
    };

    for r_type in report_types {
        let report = engine.report(repo_id, r_type, core_format)?;

        if let Some(ref path_str) = out {
            let out_path = std::path::PathBuf::from(path_str);
            let final_path = if out_path.is_dir() {
                let date_str = report
                    .generated_at
                    .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                    .unwrap_or_default();
                let ext = match report.format {
                    ReportFormat::Markdown => "md",
                    ReportFormat::Json => "json",
                    ReportFormat::Html => "html",
                };
                let type_str = match report.report_type {
                    ReportType::Audit => "audit",
                    ReportType::Trend => "trend",
                };
                let sanitized_id = report.repo.0.replace([':', '#'], "_");
                out_path.join(format!(
                    "{}-{}-{}.{}",
                    sanitized_id, type_str, date_str, ext
                ))
            } else {
                out_path.clone()
            };

            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    gitpurge_core::GitPurgeError::Config(format!(
                        "Failed to create directories: {}",
                        e
                    ))
                })?;
            }
            std::fs::write(&final_path, &report.content).map_err(|e| {
                gitpurge_core::GitPurgeError::Config(format!("Failed to write report: {}", e))
            })?;

            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "success",
                        "report_type": format!("{:?}", r_type).to_lowercase(),
                        "path": final_path.to_string_lossy(),
                    })
                );
            } else {
                println!("Report generated and saved to: {:?}", final_path);
            }
        } else {
            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "success",
                        "report_type": format!("{:?}", r_type).to_lowercase(),
                        "content": report.content,
                    })
                );
            } else {
                println!("{}", report.content);
            }
        }
    }

    Ok(())
}

/// Handle the `history` CLI command.
pub fn handle_history(
    engine: &Engine,
    repo_id: &RepoId,
    limit: u32,
    _metric: Option<String>,
    _since: Option<String>,
    json_output: bool,
) -> Result<()> {
    let history = engine.history(repo_id)?;
    let mut entries = history.entries;
    entries.reverse(); // newest first
    entries.truncate(limit as usize);

    let runs = engine.executions(repo_id, limit as usize, 0)?;

    if json_output {
        let output = serde_json::json!({
            "trends": entries,
            "executions": runs,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("Cleanup Progress & Trend History for Repo: {}", repo_id.0);
        println!(
            "--------------------------------------------------------------------------------"
        );
        println!(
            "{:<24} | {:<5} | {:<6} | {:<5} | {:<6} | {:<8}",
            "Run Date", "Total", "Active", "Stale", "Merged", "Unmerged"
        );
        println!(
            "--------------------------------------------------------------------------------"
        );
        for entry in &entries {
            #[allow(deprecated)]
            let date_str = entry
                .recorded_at
                .format(
                    &time::format_description::parse("[year]-[month]-[day] [hour]:[minute] UTC")
                        .unwrap(),
                )
                .unwrap_or_default();
            println!(
                "{:<24} | {:<5} | {:<6} | {:<5} | {:<6} | {:<8}",
                date_str,
                entry.total_branches,
                entry.active_count,
                entry.stale_count,
                entry.merged_count,
                entry.unmerged_count
            );
        }

        println!();
        println!("Past Operations Log");
        println!(
            "--------------------------------------------------------------------------------"
        );
        println!(
            "{:<16} | {:<20} | {:<8} | {:<8} | {:<7} | {:<8} | {:<10}",
            "Run ID", "Started At", "Command", "Mode", "Deleted", "Archived", "Actor"
        );
        println!(
            "--------------------------------------------------------------------------------"
        );
        for run in runs {
            #[allow(deprecated)]
            let date_str = run
                .started_at
                .format(
                    &time::format_description::parse("[year]-[month]-[day] [hour]:[minute] UTC")
                        .unwrap(),
                )
                .unwrap_or_default();
            println!(
                "{:<16} | {:<20} | {:<8} | {:<8} | {:<7} | {:<8} | {:<10}",
                run.id.chars().take(12).collect::<String>(),
                date_str.chars().take(20).collect::<String>(),
                run.command,
                run.mode,
                run.deleted_count,
                run.archived_count,
                run.actor.as_deref().unwrap_or("system")
            );
            if !run.branches.is_empty() {
                println!("  - Affected Branches:");
                for branch in run.branches {
                    println!("    * {}", branch);
                }
            }
        }
    }

    Ok(())
}

/// Handle the `history import` CLI command.
pub fn handle_history_import(
    engine: &Engine,
    path: &str,
    map: &[(String, String)],
    execute: bool,
    json_output: bool,
) -> Result<()> {
    let json_data = std::fs::read_to_string(path).map_err(|e| {
        gitpurge_core::GitPurgeError::Config(format!("Failed to read legacy JSON file: {}", e))
    })?;

    let repo_mappings = map
        .iter()
        .cloned()
        .collect::<std::collections::HashMap<_, _>>();

    let summary = engine.import_history(&json_data, &repo_mappings, execute)?;

    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "execute": execute,
                "runs_imported": summary.runs_imported,
                "metrics_imported": summary.metrics_imported,
                "skipped_runs": summary.skipped_runs,
            })
        );
    } else {
        if !execute {
            println!(
                "DRY-RUN: Showing what would be imported (run with --execute to apply changes)."
            );
        }
        println!("Legacy trend history import summary:");
        println!("  Runs parsed/imported: {}", summary.runs_imported);
        println!("  Metrics points stored: {}", summary.metrics_imported);
        println!("  Skipped (already exists): {}", summary.skipped_runs);
    }

    Ok(())
}
