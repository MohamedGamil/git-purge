//! Trend history domain models (CONVENTIONS §8, doc 10 §2.2)

use super::RepoId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Captured metrics at the end of a run or scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RunMetrics {
    /// Total count of branches.
    pub total: usize,
    /// Count of active branches.
    pub active: usize,
    /// Count of stale branches.
    pub stale: usize,
    /// Count of merged branches.
    pub merged: usize,
    /// Count of unmerged branches.
    pub unmerged: usize,
    /// Count of non-standard named branches.
    pub non_standard: usize,
    /// Count of local branches.
    pub local_count: Option<usize>,
    /// Count of remote branches.
    pub remote_count: Option<usize>,
    /// Count of protected branches.
    pub protected: Option<usize>,
    /// Count of deleted branches in this run.
    pub deleted: Option<usize>,
    /// Count of archived branches in this run.
    pub archived: Option<usize>,
    /// Count of restored branches in this run.
    pub restored: Option<usize>,
}

/// A single data point in the trend history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendEntry {
    /// When this data point was recorded.
    pub recorded_at: OffsetDateTime,
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
    /// Non-standard branches.
    pub non_standard_count: usize,
    /// Protected branches.
    pub protected_count: usize,
}

/// The full trend history for a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendHistory {
    /// The repo identifier.
    pub repo: RepoId,
    /// Ordered data points (oldest first).
    pub entries: Vec<TrendEntry>,
}

/// Delta comparison for a single metric.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDelta {
    /// The old value.
    pub old_value: usize,
    /// The new value.
    pub new_value: usize,
    /// The absolute change count.
    pub abs_change: i64,
    /// The ratio change percentage.
    pub ratio_change: f64,
}

/// Comparison table data across all six canonical metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrendComparison {
    /// Total branches delta.
    pub total: MetricDelta,
    /// Active branches delta.
    pub active: MetricDelta,
    /// Stale branches delta.
    pub stale: MetricDelta,
    /// Merged branches delta.
    pub merged: MetricDelta,
    /// Unmerged branches delta.
    pub unmerged: MetricDelta,
    /// Non-standard naming delta.
    pub non_standard: MetricDelta,
}

impl MetricDelta {
    /// Calculate delta between old and new values.
    pub fn calculate(old: usize, new: usize) -> Self {
        let abs_change = (new as i64) - (old as i64);
        let ratio_change = if old == 0 {
            0.0
        } else {
            (abs_change as f64) / (old as f64) * 100.0
        };
        Self {
            old_value: old,
            new_value: new,
            abs_change,
            ratio_change,
        }
    }
}
