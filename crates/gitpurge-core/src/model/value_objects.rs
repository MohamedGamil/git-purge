//! Supplementary value objects used across the domain model (docs/03 §10).

use serde::{Deserialize, Serialize};

/// A remote name (e.g. `"origin"`, `"upstream"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RemoteName(pub String);

/// Fields a branch list can be sorted by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    /// Sort by branch name.
    Name,
    /// Sort by last commit date.
    Date,
    /// Sort by age (duration since last commit).
    Age,
    /// Sort by author name.
    Author,
    /// Sort by merge state.
    MergeState,
    /// Sort by activity (stale/active).
    Activity,
    /// Sort by ahead count.
    Ahead,
    /// Sort by behind count.
    Behind,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SortOrder {
    /// Ascending (A→Z, oldest→newest).
    #[default]
    Ascending,
    /// Descending (Z→A, newest→oldest).
    Descending,
}

/// A filter predicate for branch lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RefFilter {
    /// Free-text search (matched against branch name).
    pub search: Option<String>,
    /// Filter by merge state.
    pub merged: Option<bool>,
    /// Filter by activity (stale vs active).
    pub stale: Option<bool>,
    /// Filter by protection status.
    pub protected: Option<bool>,
    /// Filter by scope (local/remote).
    pub scope: Option<super::BranchScope>,
    /// Sort field.
    pub sort_by: Option<SortField>,
    /// Sort direction.
    pub sort_order: Option<SortOrder>,
}
