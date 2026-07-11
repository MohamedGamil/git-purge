//! Diff and tree-view types (docs/04 §4).
//!
//! [`DiffResult`] represents the diff between two refs; [`TreeView`] represents
//! the file tree at a given ref/commit.

use serde::{Deserialize, Serialize};

use crate::model::{Oid, RefSpec};

/// A single entry in a diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Path of the file.
    pub path: String,
    /// Kind of change.
    pub kind: DiffKind,
    /// Lines added.
    pub additions: u32,
    /// Lines removed.
    pub deletions: u32,
}

/// The kind of change in a diff entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffKind {
    /// File was added.
    Added,
    /// File was deleted.
    Deleted,
    /// File was modified.
    Modified,
    /// File was renamed.
    Renamed,
    /// File was copied.
    Copied,
}

/// The result of diffing two refs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffResult {
    /// The "from" ref.
    pub from: RefSpec,
    /// The "to" ref.
    pub to: RefSpec,
    /// Per-file diff entries.
    pub entries: Vec<DiffEntry>,
    /// Total files changed.
    pub files_changed: usize,
    /// Total insertions.
    pub insertions: u32,
    /// Total deletions.
    pub deletions: u32,
}

/// A single entry in a tree view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeEntry {
    /// Path relative to the repo root.
    pub path: String,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
    /// Object id.
    pub oid: Oid,
}

/// The file tree at a given ref/commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeView {
    /// The ref this tree was read at.
    pub at: RefSpec,
    /// Entries in the tree.
    pub entries: Vec<TreeEntry>,
}
