//! Plan, actions, and execution model (docs/03 §7–§8).
//!
//! A [`Plan`] is the resolved set of [`Action`]s a command *would* take (dry-run
//! output). It is always computed before execution and is the single data structure
//! the user reviews before confirming. The plan is **never mutating** — mutation
//! happens in `Engine::execute` (which first creates a backup snapshot).

use serde::{Deserialize, Serialize};

use super::{BranchName, BranchScope, Classification, GlobPattern, Oid, RepoId, SnapshotId};

/// A ref specifier — either a branch name, a tag, or a raw SHA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefSpec {
    /// A branch name.
    Branch(BranchName),
    /// A tag name.
    Tag(String),
    /// A raw object id (SHA).
    Oid(Oid),
    /// A symbolic ref like HEAD.
    Symbolic(String),
}

/// Options controlling the `scan` pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScanOptions {
    /// Restrict scan to local or remote branches (default: both).
    pub scope: Option<BranchScope>,
    /// Override age threshold for this scan only.
    pub age_override: Option<String>,
    /// Additional excludes layered on top of policy.
    pub excludes: Vec<GlobPattern>,
    /// Include branches that would normally be excluded by policy.
    pub include_all: bool,
}

/// Result of a scan — classified branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    /// The repo that was scanned.
    pub repo: RepoId,
    /// All classified branches.
    pub classifications: Vec<Classification>,
    /// Total branch count before filtering.
    pub total_branches: usize,
    /// Branches excluded by policy/filters.
    pub excluded_count: usize,
}

/// What kind of action the plan prescribes for a branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    /// Delete the branch (local, remote, or both).
    Delete,
    /// Archive the branch (merge into a legacy branch via ours/theirs).
    Archive,
    /// Restore a branch from a snapshot.
    Restore,
}

/// A single action in a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// What to do.
    pub kind: ActionKind,
    /// The branch targeted.
    pub branch: BranchName,
    /// Local, remote, or both.
    pub scope: BranchScope,
    /// The remote name, if the action targets a remote branch.
    pub remote: Option<String>,
    /// The classification that motivated this action.
    pub classification: Classification,
    /// Human-readable reason this action was proposed.
    pub reason: String,
}

/// Filtering criteria for plan resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActionFilter {
    /// Which action kind to plan for.
    pub kind: Option<ActionKind>,
    /// Only include merged branches.
    pub merged_only: bool,
    /// Include unmerged branches (requires stronger confirmation).
    pub include_unmerged: bool,
    /// Branch name glob includes.
    pub include_globs: Vec<GlobPattern>,
    /// Branch name glob excludes.
    pub exclude_globs: Vec<GlobPattern>,
    /// Override age threshold.
    pub age_override: Option<String>,
    /// Specific branches to target (overrides globs).
    pub specific_branches: Vec<BranchName>,
}

/// Whether to actually mutate or just preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecMode {
    /// Show what would happen (the default — SAFE-01).
    #[default]
    DryRun,
    /// Actually perform the mutations (requires explicit opt-in).
    Execute,
}

/// A resolved plan — the set of actions a command would take.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// The repo this plan targets.
    pub repo: RepoId,
    /// The resolved actions.
    pub actions: Vec<Action>,
    /// Branches that were considered but excluded (protected, filtered out).
    pub skipped_count: usize,
    /// Human summary of the plan.
    pub summary: String,
}

/// How to restore a ref from a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreSpec {
    /// The branch to restore.
    pub branch: BranchName,
    /// Restore as a tag instead of a branch.
    pub as_tag: bool,
    /// Override the name (default: original name).
    pub target_name: Option<String>,
    /// Force overwrite if the target already exists (default: false — SAFE-06).
    pub force: bool,
}

/// The outcome of a restore operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreOutcome {
    /// The snapshot the ref was restored from.
    pub snapshot: SnapshotId,
    /// The branch that was restored.
    pub branch: BranchName,
    /// The ref that was created.
    pub created_ref: String,
    /// Whether it was created as a tag.
    pub as_tag: bool,
    /// The tip commit of the restored ref.
    pub tip: Oid,
}

/// Per-action result within a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionResult {
    /// The action succeeded.
    Success {
        /// The action that was performed.
        action: Action,
    },
    /// The action failed.
    Failed {
        /// The action that was attempted.
        action: Action,
        /// Error message (never contains secrets — SAFE-07).
        error: String,
    },
    /// The action was skipped (dry-run).
    Skipped {
        /// The action that was skipped.
        action: Action,
    },
}

/// A recorded execution and its metrics (feeds trend history, R7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunReport {
    /// Unique identifier for this run.
    pub id: String,
    /// When the run started.
    pub started_at: time::OffsetDateTime,
    /// The repo the run targeted.
    pub repo: RepoId,
    /// The execution mode.
    pub mode: ExecMode,
    /// The pre-op snapshot id (if backup was created — SAFE-04).
    pub snapshot: Option<SnapshotId>,
    /// Per-action results.
    pub results: Vec<ActionResult>,
    /// Count of successful actions.
    pub success_count: usize,
    /// Count of failed actions.
    pub failure_count: usize,
    /// Count of skipped actions (dry-run).
    pub skipped_count: usize,
    /// The command name (e.g., 'scan', 'delete', 'archive').
    pub command: String,
    /// The metrics captured at the end of this run.
    pub metrics: Option<super::RunMetrics>,
    /// Optional snapshot of all branch classifications at the time of the run.
    pub branch_snapshots: Option<Vec<super::Classification>>,
}
