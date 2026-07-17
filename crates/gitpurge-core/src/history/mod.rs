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

pub use sqlite::SqliteHistoryStore;

use crate::error::Result;
use crate::model::{
    RepoId, Repository, RunRecord, RunReport, Snapshot, SnapshotId, TrendEntry, TrendHistory,
};

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
}
