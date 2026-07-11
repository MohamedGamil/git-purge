#![allow(deprecated)]
// reporting.rs — CLI reporting command handlers (Phase P5)

use gitpurge_core::{Engine, Result, report::{ReportFormat, ReportType}, model::RepoId};

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
                let date_str = report.generated_at.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap_or_default();
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
                out_path.join(format!("{}-{}-{}.{}", sanitized_id, type_str, date_str, ext))
            } else {
                out_path.clone()
            };

            if let Some(parent) = final_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    gitpurge_core::GitPurgeError::Config(format!("Failed to create directories: {}", e))
                })?;
            }
            std::fs::write(&final_path, &report.content).map_err(|e| {
                gitpurge_core::GitPurgeError::Config(format!("Failed to write report: {}", e))
            })?;

            if json_output {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "report_type": format!("{:?}", r_type).to_lowercase(),
                    "path": final_path.to_string_lossy(),
                }));
            } else {
                println!("Report generated and saved to: {:?}", final_path);
            }
        } else {
            if json_output {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "report_type": format!("{:?}", r_type).to_lowercase(),
                    "content": report.content,
                }));
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

    if json_output {
        println!("{}", serde_json::to_string_pretty(&entries).unwrap());
    } else {
        println!("Cleanup Progress & Trend History for Repo: {}", repo_id.0);
        println!("--------------------------------------------------------------------------------");
        println!("{:<24} | {:<5} | {:<6} | {:<5} | {:<6} | {:<8}", "Run Date", "Total", "Active", "Stale", "Merged", "Unmerged");
        println!("--------------------------------------------------------------------------------");
        for entry in entries {
            let date_str = entry.recorded_at.format(&time::format_description::parse("[year]-[month]-[day] [hour]:[minute] UTC").unwrap()).unwrap_or_default();
            println!("{:<24} | {:<5} | {:<6} | {:<5} | {:<6} | {:<8}",
                date_str,
                entry.total_branches,
                entry.active_count,
                entry.stale_count,
                entry.merged_count,
                entry.unmerged_count
            );
        }
    }

    Ok(())
}
