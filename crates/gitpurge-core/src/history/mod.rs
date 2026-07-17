//! History / trend storage port (docs/02 §3, docs/10 reporting-and-history).
//!
//! `HistoryStore` abstracts the SQLite trend database so the service layer
//! (and tests) don't depend on `rusqlite` directly.

/// DDL schema migrations.
pub mod migrate;
/// SQLite adapter for history/trends.
pub mod sqlite;
/// Trend calculations and comparison algorithms.
pub mod trends;
/// Legacy history data import.
pub mod import;

pub use sqlite::SqliteHistoryStore;

use crate::error::Result;
use crate::model::{
    RepoId, Repository, RunRecord, RunReport, Snapshot, SnapshotId, TrendEntry, TrendHistory,
};

/// Summary of historical trend data import operations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImportSummary {
    /// Number of runs imported.
    pub runs_imported: usize,
    /// Number of metrics records imported (after deduplication).
    pub metrics_imported: usize,
    /// Number of skipped runs that already existed.
    pub skipped_runs: usize,
}

/// Port for run recording and trend queries.
///
/// Implementations must be `Send + Sync` for shared `Engine` access.
pub trait HistoryStore: Send + Sync + std::fmt::Debug {
    /// Save or update a repository's metadata in the history store.
    fn save_repo(&self, repo: &Repository) -> Result<()>;

    /// Record a completed run.
    fn record_run(&self, report: &RunReport) -> Result<()>;

    /// Fetch the trend history for a repo.
    fn get_history(&self, repo: &RepoId) -> Result<TrendHistory>;

    /// Fetch the most recent N entries.
    fn get_recent(&self, repo: &RepoId, limit: usize) -> Result<Vec<TrendEntry>>;

    /// Fetch past executions for a repository.
    fn get_runs(&self, repo: &RepoId, limit: usize, offset: usize) -> Result<Vec<RunRecord>>;

    /// Fetch all branch classifications captured in a specific run.
    fn get_run_classifications(&self, run_id: &str) -> Result<Vec<crate::model::Classification>>;

    /// Fetch a run record by its ID.
    fn get_run_record(&self, run_id: &str) -> Result<Option<RunRecord>>;

    /// Save snapshot metadata.
    fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()>;

    /// List all snapshots for a repo, newest first.
    fn list_snapshots(&self, repo: &RepoId) -> Result<Vec<Snapshot>>;

    /// Get snapshot details by ID.
    fn get_snapshot(&self, id: &SnapshotId) -> Result<Option<Snapshot>>;

    /// Delete snapshot metadata.
    fn delete_snapshot(&self, id: &SnapshotId) -> Result<()>;

    /// Import legacy JSON history data.
    fn import_legacy_json(
        &self,
        json_data: &str,
        repo_mappings: &std::collections::HashMap<String, String>,
        execute: bool,
    ) -> Result<ImportSummary>;
}

/// In-memory fake for tests.
#[derive(Debug, Default)]
pub struct FakeHistoryStore {
    snapshots: std::sync::Mutex<std::collections::HashMap<SnapshotId, Snapshot>>,
    runs: std::sync::Mutex<std::collections::HashMap<String, RunReport>>,
}

impl FakeHistoryStore {
    /// Create a new empty FakeHistoryStore.
    pub fn new() -> Self {
        Self {
            snapshots: std::sync::Mutex::new(std::collections::HashMap::new()),
            runs: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl HistoryStore for FakeHistoryStore {
    fn save_repo(&self, _repo: &Repository) -> Result<()> {
        Ok(())
    }

    fn record_run(&self, report: &RunReport) -> Result<()> {
        let mut runs = self.runs.lock().unwrap();
        runs.insert(report.id.clone(), report.clone());
        Ok(())
    }

    fn get_history(&self, repo: &RepoId) -> Result<TrendHistory> {
        Ok(TrendHistory {
            repo: repo.clone(),
            entries: Vec::new(),
        })
    }

    fn get_recent(&self, _repo: &RepoId, _limit: usize) -> Result<Vec<TrendEntry>> {
        Ok(Vec::new())
    }

    fn get_runs(&self, _repo: &RepoId, _limit: usize, _offset: usize) -> Result<Vec<RunRecord>> {
        Ok(Vec::new())
    }

    fn get_run_classifications(&self, run_id: &str) -> Result<Vec<crate::model::Classification>> {
        let runs = self.runs.lock().unwrap();
        if let Some(report) = runs.get(run_id) {
            Ok(report.branch_snapshots.clone().unwrap_or_default())
        } else {
            Ok(Vec::new())
        }
    }

    fn get_run_record(&self, run_id: &str) -> Result<Option<RunRecord>> {
        let runs = self.runs.lock().unwrap();
        if let Some(report) = runs.get(run_id) {
            let started_at = report.started_at;
            let finished_at = Some(time::OffsetDateTime::now_utc());
            Ok(Some(RunRecord {
                id: report.id.clone(),
                command: report.command.clone(),
                mode: match report.mode {
                    crate::model::ExecMode::DryRun => "dry-run".to_string(),
                    crate::model::ExecMode::Execute => "execute".to_string(),
                },
                started_at,
                finished_at,
                snapshot_id: report.snapshot.as_ref().map(|s| s.0.clone()),
                actor: Some("system".to_string()),
                deleted_count: report.metrics.as_ref().and_then(|m| m.deleted).unwrap_or(0),
                archived_count: report
                    .metrics
                    .as_ref()
                    .and_then(|m| m.archived)
                    .unwrap_or(0),
                branches: report
                    .branch_snapshots
                    .as_ref()
                    .map(|bs| bs.iter().map(|b| b.branch.0.clone()).collect())
                    .unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let mut snaps = self.snapshots.lock().unwrap();
        snaps.insert(snapshot.id.clone(), snapshot.clone());
        Ok(())
    }

    fn list_snapshots(&self, repo: &RepoId) -> Result<Vec<Snapshot>> {
        let snaps = self.snapshots.lock().unwrap();
        let mut result: Vec<Snapshot> = snaps
            .values()
            .filter(|s| s.repo == *repo)
            .cloned()
            .collect();
        result.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(result)
    }

    fn get_snapshot(&self, id: &SnapshotId) -> Result<Option<Snapshot>> {
        let snaps = self.snapshots.lock().unwrap();
        Ok(snaps.get(id).cloned())
    }

    fn delete_snapshot(&self, id: &SnapshotId) -> Result<()> {
        let mut snaps = self.snapshots.lock().unwrap();
        snaps.remove(id);
        Ok(())
    }

    fn import_legacy_json(
        &self,
        json_data: &str,
        repo_mappings: &std::collections::HashMap<String, String>,
        execute: bool,
    ) -> Result<ImportSummary> {
        let legacy_data = crate::history::import::parse_legacy_json(json_data)?;
        let mut runs_imported = 0;
        let mut metrics_imported = 0;
        let mut skipped_runs = 0;

        for (legacy_repo_name, mut entries) in legacy_data {
            let repo_id_str = repo_mappings
                .get(&legacy_repo_name)
                .cloned()
                .unwrap_or_else(|| legacy_repo_name.clone());
            let repo_id = RepoId(repo_id_str);

            entries.sort_by(|a, b| {
                a.timestamp
                    .partial_cmp(&b.timestamp)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut last_metrics_hash: Option<String> = None;

            for entry in entries {
                let started_at = crate::history::import::parse_legacy_timestamp(entry.timestamp);

                let mut exists = false;
                {
                    let runs = self.runs.lock().unwrap();
                    for r in runs.values() {
                        if r.repo == repo_id && r.started_at == started_at && r.command == "scan" {
                            exists = true;
                            break;
                        }
                    }
                }

                if exists {
                    skipped_runs += 1;
                    continue;
                }

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
                    let report = RunReport {
                        id: run_id.clone(),
                        started_at,
                        repo: repo_id.clone(),
                        mode: crate::model::ExecMode::Execute,
                        snapshot: None,
                        results: Vec::new(),
                        success_count: 0,
                        failure_count: 0,
                        skipped_count: 0,
                        command: "scan".to_string(),
                        metrics: Some(crate::model::RunMetrics {
                            total: metrics.total,
                            active: metrics.active,
                            stale: metrics.stale,
                            merged: metrics.merged,
                            unmerged: metrics.unmerged,
                            non_standard: metrics.non_standard,
                            local_count: None,
                            remote_count: None,
                            protected: None,
                            deleted: Some(0),
                            archived: Some(0),
                            restored: Some(0),
                        }),
                        branch_snapshots: None,
                    };
                    self.runs.lock().unwrap().insert(run_id, report);
                    runs_imported += 1;

                    if last_metrics_hash.as_ref() != Some(&metrics_hash) {
                        metrics_imported += 1;
                        last_metrics_hash = Some(metrics_hash);
                    }
                } else {
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
}
