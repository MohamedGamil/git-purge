//! History / trend storage port (docs/02 §3, docs/10 reporting-and-history).
//!
//! `HistoryStore` abstracts the SQLite trend database so the service layer
//! (and tests) don't depend on `rusqlite` directly.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::{RepoId, RunReport};

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
}

/// In-memory fake for tests.
#[derive(Debug, Default)]
pub struct FakeHistoryStore {
    // TODO(P5): add fields for canned history data.
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
}
