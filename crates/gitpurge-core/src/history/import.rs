//! Legacy history import implementation (Phase 12, P12-T3).

use crate::error::Result;
use crate::history::ImportSummary;
use crate::model::RepoId;
use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;
use time::OffsetDateTime;

/// An entry representing a single execution scan in the legacy history format.
#[derive(Debug, serde::Deserialize, Clone)]
pub struct LegacyEntry {
    /// Unix timestamp of the recorded execution.
    pub timestamp: f64,
    /// Associated commit hash when the scan was executed.
    pub commit: Option<String>,
    /// Human-readable date string of the execution.
    pub date_str: Option<String>,
    /// The six canonical branch classification metrics.
    pub metrics: LegacyMetrics,
}

/// The six canonical branch metrics recorded in legacy format.
#[derive(Debug, serde::Deserialize, Clone)]
pub struct LegacyMetrics {
    /// Total branches count.
    pub total: usize,
    /// Active branches count.
    pub active: usize,
    /// Stale branches count.
    pub stale: usize,
    /// Merged branches count.
    pub merged: usize,
    /// Unmerged branches count.
    pub unmerged: usize,
    /// Branches violating the naming policy.
    pub non_standard: usize,
}

/// Parse the legacy branch history JSON structure.
pub fn parse_legacy_json(json_data: &str) -> Result<HashMap<String, Vec<LegacyEntry>>> {
    serde_json::from_str(json_data)
        .map_err(|e| crate::GitPurgeError::Config(format!("Failed to parse legacy JSON: {}", e)))
}

/// Helper to parse a unix timestamp f64 into an OffsetDateTime.
pub fn parse_legacy_timestamp(timestamp: f64) -> OffsetDateTime {
    // Robustly handle both seconds and milliseconds (detect if > 10,000,000,000)
    let timestamp_secs = if timestamp > 10_000_000_000.0 {
        timestamp / 1000.0
    } else {
        timestamp
    };
    let seconds = timestamp_secs.floor() as i64;
    let nanos = ((timestamp_secs - timestamp_secs.floor()) * 1_000_000_000.0) as u32;
    OffsetDateTime::from_unix_timestamp(seconds)
        .map(|t| t + time::Duration::nanoseconds(nanos as i64))
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
}

/// Execute import operations on a database connection.
pub fn import_legacy_db(
    conn: &Connection,
    legacy_data: HashMap<String, Vec<LegacyEntry>>,
    repo_mappings: &HashMap<String, String>,
    execute: bool,
) -> Result<ImportSummary> {
    let mut runs_imported = 0;
    let mut metrics_imported = 0;
    let mut skipped_runs = 0;

    let now_str = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default();

    for (legacy_repo_name, mut entries) in legacy_data {
        // Resolve RepoId
        let repo_id_str = repo_mappings
            .get(&legacy_repo_name)
            .cloned()
            .unwrap_or_else(|| legacy_repo_name.clone());
        let repo_id = RepoId(repo_id_str);

        // Sort chronologically by timestamp
        entries.sort_by(|a, b| {
            a.timestamp
                .partial_cmp(&b.timestamp)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Ensure the repository exists in database if executing
        if execute {
            conn.execute(
                "INSERT OR IGNORE INTO repos (id, canonical_url, local_path, path_hash, display_name, default_branch, created_at, tombstoned_at)
                 VALUES (?1, NULL, NULL, 'imported', ?1, 'main', ?2, NULL);",
                (&repo_id.0, &now_str),
            ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to register imported repo: {}", e)))?;
        }

        // Cache the latest metrics hash to avoid querying DB constantly
        let mut last_metrics_hash: Option<String> = if execute {
            conn.query_row(
                "SELECT metrics_hash FROM metrics WHERE repo_id = ?1 ORDER BY captured_at DESC LIMIT 1;",
                [&repo_id.0],
                |row| row.get(0)
            ).optional().map_err(|e| crate::GitPurgeError::Config(format!("Failed to query latest metrics: {}", e)))?
        } else {
            None
        };

        for entry in entries {
            let started_at = parse_legacy_timestamp(entry.timestamp);
            let started_at_str = started_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default();

            // Check if run already exists
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM runs WHERE repo_id = ?1 AND started_at = ?2 AND command = 'scan');",
                (&repo_id.0, &started_at_str),
                |row| row.get(0),
            ).unwrap_or(false);

            if exists {
                skipped_runs += 1;
                continue;
            }

            // Compute metrics hash
            let metrics = &entry.metrics;
            let hash_str = format!(
                "{}:{}:{}:{}:{}:{}",
                metrics.total,
                metrics.active,
                metrics.stale,
                metrics.merged,
                metrics.unmerged,
                metrics.non_standard
            );
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            hash_str.hash(&mut hasher);
            let metrics_hash = format!("{:016x}", hasher.finish());

            if execute {
                let run_id = ulid::Ulid::new().to_string();
                let note = format!(
                    "Legacy Import. Commit: {}, Date: {}",
                    entry.commit.as_deref().unwrap_or("none"),
                    entry.date_str.as_deref().unwrap_or("none")
                );

                conn.execute(
                    "INSERT INTO runs (id, repo_id, command, mode, started_at, finished_at, snapshot_id, age_threshold, actor, tool_version, exit_code, note)
                     VALUES (?1, ?2, 'scan', 'execute', ?3, ?3, NULL, NULL, 'legacy-import', ?4, NULL, ?5);",
                    (
                        &run_id,
                        &repo_id.0,
                        &started_at_str,
                        env!("CARGO_PKG_VERSION"),
                        &note,
                    ),
                ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to insert run: {}", e)))?;

                runs_imported += 1;

                if last_metrics_hash.as_ref() != Some(&metrics_hash) {
                    conn.execute(
                        "INSERT INTO metrics (
                            run_id, repo_id, captured_at, total, active, stale, merged, unmerged, non_standard,
                            local_count, remote_count, protected, deleted, archived, restored, metrics_hash
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, NULL, NULL, 0, 0, 0, ?10);",
                        (
                            &run_id,
                            &repo_id.0,
                            &started_at_str,
                            metrics.total as i64,
                            metrics.active as i64,
                            metrics.stale as i64,
                            metrics.merged as i64,
                            metrics.unmerged as i64,
                            metrics.non_standard as i64,
                            &metrics_hash,
                        ),
                    ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to insert metrics: {}", e)))?;

                    metrics_imported += 1;
                    last_metrics_hash = Some(metrics_hash);
                }
            } else {
                // Dry run
                runs_imported += 1;
                if last_metrics_hash.as_ref() != Some(&metrics_hash) {
                    metrics_imported += 1;
                    last_metrics_hash = Some(metrics_hash);
                }
            }
        }
    }

    Ok(ImportSummary {
        runs_imported,
        metrics_imported,
        skipped_runs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::FakeHistoryStore;
    use crate::history::HistoryStore;

    #[test]
    fn test_parse_legacy_timestamp() {
        let dt = parse_legacy_timestamp(1625097600.0);
        assert_eq!(dt.unix_timestamp(), 1625097600);

        let dt_ms = parse_legacy_timestamp(1625097600000.0);
        assert_eq!(dt_ms.unix_timestamp(), 1625097600);
    }

    #[test]
    fn test_import_legacy_json_dry_run_and_execute() {
        let json_data = r#"{
            "backend": [
                {
                    "timestamp": 1625097600.0,
                    "commit": "abc",
                    "date_str": "2021-07-01",
                    "metrics": {
                        "total": 10,
                        "active": 4,
                        "stale": 6,
                        "merged": 3,
                        "unmerged": 7,
                        "non_standard": 1
                    }
                },
                {
                    "timestamp": 1625097700.0,
                    "commit": "def",
                    "date_str": "2021-07-01",
                    "metrics": {
                        "total": 10,
                        "active": 4,
                        "stale": 6,
                        "merged": 3,
                        "unmerged": 7,
                        "non_standard": 1
                    }
                }
            ]
        }"#;

        let store = FakeHistoryStore::new();
        let mappings = HashMap::new();

        let summary = store
            .import_legacy_json(json_data, &mappings, false)
            .unwrap();
        assert_eq!(summary.runs_imported, 2);
        assert_eq!(summary.metrics_imported, 1);
        assert_eq!(summary.skipped_runs, 0);

        let summary2 = store
            .import_legacy_json(json_data, &mappings, true)
            .unwrap();
        assert_eq!(summary2.runs_imported, 2);
        assert_eq!(summary2.metrics_imported, 1);
        assert_eq!(summary2.skipped_runs, 0);

        let summary3 = store
            .import_legacy_json(json_data, &mappings, true)
            .unwrap();
        assert_eq!(summary3.runs_imported, 0);
        assert_eq!(summary3.metrics_imported, 0);
        assert_eq!(summary3.skipped_runs, 2);
    }
}
