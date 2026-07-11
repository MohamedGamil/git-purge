//! Snapshots & restore points — the backup model (docs/03 §6, CONVENTIONS §6).
//!
//! A [`Snapshot`] does **not** clone the repo; it writes captured refs into a
//! namespaced ref inside the shared bare mirror
//! (`refs/gitpurge/backups/<snapshot-id>/<original-ref-path>`), so N snapshots cost
//! ~O(changed objects), not O(N × repo size).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{BranchName, MergeState, Oid, RepoId};

/// A snapshot id — ULID/uuid, sortable by creation time.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

/// What triggered a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotTrigger {
    /// Explicit `backup create`.
    Manual,
    /// Automatic pre-op backup before a delete.
    PreDelete,
    /// Automatic pre-op backup before an archive.
    PreArchive,
    /// Scheduled backup.
    Scheduled,
}

/// Metadata for one captured ref (CONVENTIONS §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRef {
    /// Original short branch name.
    pub branch: BranchName,
    /// Where it lived, for faithful restore.
    pub original_full_ref: String,
    /// `refs/gitpurge/backups/<id>/<orig>`.
    pub backup_ref: String,
    /// Tip commit SHA at capture.
    pub tip: Oid,
    /// Commits reachable — proves "content backed up".
    pub commit_count: u64,
    /// Upstream at capture time (e.g. `origin/feature/x`).
    pub upstream: Option<String>,
    /// Merge status frozen at capture time.
    pub merged_at_capture: MergeState,
}

/// A point-in-time capture of a repo's refs. This IS the restore point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot id.
    pub id: SnapshotId,
    /// The repo it belongs to.
    pub repo: RepoId,
    /// Creation time.
    pub created_at: OffsetDateTime,
    /// What triggered it.
    pub trigger: SnapshotTrigger,
    /// One entry per captured branch/tag.
    pub refs: Vec<SnapshotRef>,
    /// Whether objects were readable back (backup-before-destroy verification).
    pub verified: bool,
    /// Path of `snapshot.json` inside the bare mirror.
    pub manifest_path: PathBuf,
}

/// A restore point is simply a snapshot referenced for restoration. The alias exists
/// so docs/UI can speak of "restore points" (CONVENTIONS §8).
pub type RestorePoint = Snapshot;

/// Options controlling a backup snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BackupOptions {
    /// What triggered this snapshot (defaults to `Manual` when unset).
    pub trigger: Option<SnapshotTrigger>,
    /// Verify object readability after capture (backup-before-destroy). Default true.
    pub verify: bool,
    /// Restrict capture to these branches; empty ⇒ capture all eligible refs.
    pub only_branches: Vec<BranchName>,
}

/// Retention policy for snapshots (docs/08 §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Keep the N newest snapshots.
    pub keep_last: Option<usize>,
    /// Keep snapshots newer than a cutoff duration.
    pub keep_within: Option<std::time::Duration>,
    /// Triggers that should always be kept.
    pub keep_triggers: Vec<SnapshotTrigger>,
    /// Absolute floor of snapshots to keep to prevent emptying the store.
    pub min_keep: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_last: Some(10),
            keep_within: None,
            keep_triggers: Vec::new(),
            min_keep: 1,
        }
    }
}

/// The outcome of a snapshot prune operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruneReport {
    /// The IDs of snapshots that were deleted.
    pub pruned_snapshots: Vec<SnapshotId>,
    /// Estimated disk space reclaimed (bytes).
    pub space_reclaimed_bytes: u64,
}
