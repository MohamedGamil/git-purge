//! History / trend storage port (docs/02 §3, docs/10 reporting-and-history).
//!
//! `HistoryStore` abstracts the SQLite trend database so the service layer
//! (and tests) don't depend on `rusqlite` directly.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::{RepoId, RunReport, Snapshot, SnapshotId};

/// A single data point in the trend history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendEntry {
    /// When this data point was recorded.
    pub recorded_at: time::OffsetDateTime,
    /// Total branches at time of scan.
    pub total_branches: usize,
    /// Merged branches.
    pub merged_count: usize,
    /// Unmerged branches.
    pub unmerged_count: usize,
    /// Stale branches.
    pub stale_count: usize,
    /// Active branches.
    pub active_count: usize,
    /// Branches deleted in this run.
    pub deleted_count: usize,
    /// Branches archived in this run.
    pub archived_count: usize,
}

/// The full trend history for a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendHistory {
    /// The repo.
    pub repo: RepoId,
    /// Ordered data points (oldest first).
    pub entries: Vec<TrendEntry>,
}

/// Port for run recording and trend queries.
///
/// Implementations must be `Send + Sync` for shared `Engine` access.
pub trait HistoryStore: Send + Sync + std::fmt::Debug {
    /// Record a completed run.
    fn record_run(&self, report: &RunReport) -> Result<()>;

    /// Fetch the trend history for a repo.
    fn get_history(&self, repo: &RepoId) -> Result<TrendHistory>;

    /// Fetch the most recent N entries.
    fn get_recent(&self, repo: &RepoId, limit: usize) -> Result<Vec<TrendEntry>>;

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
}

impl FakeHistoryStore {
    /// Create a new empty FakeHistoryStore.
    pub fn new() -> Self {
        Self {
            snapshots: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl HistoryStore for FakeHistoryStore {
    fn record_run(&self, _report: &RunReport) -> Result<()> {
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
