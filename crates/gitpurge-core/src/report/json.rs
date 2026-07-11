//! JSON report generation (doc 10 §5)

use crate::error::Result;
use crate::model::{Repository, ScanResult, TrendHistory};
use serde_json::json;

/// Generate a structured JSON report payload.
pub fn generate_json_report(
    repo: &Repository,
    scan: &ScanResult,
    history: Option<&TrendHistory>,
    generated_at: time::OffsetDateTime,
) -> Result<String> {
    let generated_at_str = generated_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default();

    let total = scan.total_branches;
    let active = scan.classifications.iter().filter(|c| matches!(c.activity, crate::model::Activity::Active)).count();
    let stale = scan.classifications.iter().filter(|c| matches!(c.activity, crate::model::Activity::Stale)).count();
    let merged = scan.classifications.iter().filter(|c| matches!(c.merge_state, crate::model::MergeState::Merged)).count();
    let unmerged = scan.classifications.iter().filter(|c| matches!(c.merge_state, crate::model::MergeState::Unmerged)).count();
    let non_standard = scan.classifications.iter().filter(|c| !matches!(c.naming, crate::model::NamingVerdict::Standard | crate::model::NamingVerdict::Exempt { .. })).count();

    let payload = json!({
        "schema_version": "1",
        "generated_at": generated_at_str,
        "repo": {
            "id": repo.id.0,
            "display_name": repo.display_name,
            "local_path": repo.local_path.as_ref().map(|p| p.to_string_lossy()),
            "remote_url": repo.remote_url.as_ref().map(|u| &u.raw),
        },
        "metrics": {
            "total": total,
            "active": active,
            "stale": stale,
            "merged": merged,
            "unmerged": unmerged,
            "non_standard": non_standard,
        },
        "classifications": scan.classifications,
        "history": history.map(|h| &h.entries),
    });

    serde_json::to_string_pretty(&payload)
        .map_err(|e| crate::GitPurgeError::Config(format!("Failed to serialize JSON report: {}", e)))
}
