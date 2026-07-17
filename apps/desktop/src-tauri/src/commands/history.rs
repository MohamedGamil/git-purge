use tauri::State;

use gitpurge_core::model::RepoId;

use super::{format_datetime, map_error, SerializableError};
use crate::AppState;

#[tauri::command]
pub async fn report_generate(
    state: State<'_, AppState>,
    repo_id: String,
    format: String,
    report_type: Option<String>,
) -> Result<serde_json::Value, SerializableError> {
    let engine = &state.engine;
    let fmt = match format.as_str() {
        "json" => gitpurge_core::report::ReportFormat::Json,
        "html" => gitpurge_core::report::ReportFormat::Html,
        _ => gitpurge_core::report::ReportFormat::Markdown,
    };

    let rep_type = match report_type.as_deref() {
        Some("trend") => gitpurge_core::report::ReportType::Trend,
        _ => gitpurge_core::report::ReportType::Audit,
    };

    let report = engine
        .report(&RepoId(repo_id), rep_type, fmt)
        .map_err(map_error)?;

    Ok(serde_json::json!({
        "content": report.content,
        "generatedAt": format_datetime(report.generated_at),
    }))
}

#[tauri::command]
pub async fn history_get(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<serde_json::Value, SerializableError> {
    let engine = &state.engine;
    let history = engine.history(&RepoId(repo_id)).map_err(map_error)?;

    let serializable_entries: Vec<serde_json::Value> = history
        .entries
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "recordedAt": format_datetime(entry.recorded_at),
                "totalBranches": entry.total_branches,
                "activeCount": entry.active_count,
                "staleCount": entry.stale_count,
                "mergedCount": entry.merged_count,
                "unmergedCount": entry.unmerged_count,
                "deletedCount": entry.deleted_count,
                "archivedCount": entry.archived_count,
                "nonStandardCount": entry.non_standard_count,
            })
        })
        .collect();

    Ok(serde_json::json!(serializable_entries))
}

#[tauri::command]
pub async fn history_runs_get(
    state: State<'_, AppState>,
    repo_id: String,
    limit: usize,
    offset: usize,
) -> Result<serde_json::Value, SerializableError> {
    let engine = &state.engine;
    let runs = engine
        .executions(&RepoId(repo_id), limit, offset)
        .map_err(map_error)?;

    let serializable_runs: Vec<serde_json::Value> = runs
        .into_iter()
        .map(|run| {
            serde_json::json!({
                "id": run.id,
                "command": run.command,
                "mode": run.mode,
                "startedAt": format_datetime(run.started_at),
                "finishedAt": run.finished_at.map(format_datetime),
                "snapshotId": run.snapshot_id,
                "actor": run.actor,
                "deletedCount": run.deleted_count,
                "archivedCount": run.archived_count,
                "branches": run.branches,
            })
        })
        .collect();

    Ok(serde_json::json!(serializable_runs))
}
