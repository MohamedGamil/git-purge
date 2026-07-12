//! Classification — the computed facets of a branch (docs/03 §3).
//!
//! [`Classification`] is a pure function of a `Branch` + a [`super::Policy`] + a clock,
//! so it is deterministic and unit-testable against fixture repos. Every facet is a
//! filterable and sortable dimension (docs/03 §3.2).

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{BranchName, BranchScope, Commit, Upstream};

/// A parsed age threshold. Accepts human input like `"1 year ago"`, `"6 months"`,
/// `"90d"` (mirrors the bash scripts' `--age "1 year ago"`), normalized to a duration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgeThreshold {
    /// The raw string the user supplied.
    pub raw: String,
    /// The normalized duration.
    pub duration: Duration,
}

/// The computed facets of a branch — the heart of `scan`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    /// The branch these facets describe.
    pub branch: BranchName,
    /// Local or remote.
    pub scope: BranchScope,
    /// The remote name, if this is a remote branch.
    #[serde(default)]
    pub remote: Option<String>,
    /// The upstream relationship details, if any (for local branches).
    #[serde(default)]
    pub upstream: Option<Upstream>,
    /// Merge state relative to the default branch.
    pub merge_state: MergeState,
    /// Stale vs active by age threshold.
    pub activity: Activity,
    /// `now - tip.commit_date`.
    pub age: Duration,
    /// Protection verdict.
    pub protection: Protection,
    /// Naming-policy verdict.
    pub naming: NamingVerdict,
    /// Ahead/behind + gone-upstream facet.
    pub tracking: TrackingFacet,
    /// Denormalized tip commit for display/sort.
    pub tip: Commit,
    /// Suggested action — advisory only.
    pub recommendation: Recommendation,
}

/// Merge state of a branch relative to the resolved default branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeState {
    /// tip is an ancestor of the default branch (`merge-base --is-ancestor`).
    Merged,
    /// tip is not an ancestor of the default branch.
    Unmerged,
    /// Default branch could not be resolved — treated as UNMERGED for safety
    /// (never auto-delete on `Unknown`).
    Unknown,
}

/// Activity facet: staleness by age threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    /// `age >= policy.age_threshold`.
    Stale,
    /// `age < policy.age_threshold`.
    Active,
}

/// Whether a branch is protected, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protection {
    /// Not protected.
    Unprotected,
    /// Protected, with a machine + human reason for auditability.
    Protected {
        /// Why this ref is protected.
        reason: ProtectionReason,
    },
}

/// The reason a ref is protected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectionReason {
    /// It *is* the resolved default branch.
    DefaultBranch,
    /// A well-known name: main/master/develop/staging/production/HEAD.
    WellKnown(String),
    /// Matched `Policy.protected_names`.
    UserListed(String),
    /// Matched a `Policy.protected_globs` pattern.
    GlobMatch(String),
    /// Currently checked out.
    IsHead,
    /// A tag reached through a branch operation.
    IsTag,
}

/// Naming-policy verdict (generalizes the legacy `is_standard_branch`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamingVerdict {
    /// Matched an allowed rule.
    Standard,
    /// Matched an explicit allowed exception.
    Exempt {
        /// The exception rule that matched.
        rule: String,
    },
    /// Did not match any allowed rule.
    NonStandard {
        /// Why it is non-standard.
        reason: NamingViolation,
    },
}

/// A specific naming violation with a human-facing reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamingViolation {
    /// No `/` category prefix — e.g. "wip", "temp".
    NoCategoryPrefix,
    /// Known prefix, bad case/version format.
    WrongPrefixFormat {
        /// The offending prefix.
        prefix: String,
    },
    /// Non-standard prefix — e.g. "bugfix/", "task/".
    NonStandardPrefix {
        /// The offending prefix.
        prefix: String,
    },
    /// Anything else.
    UnknownPrefix {
        /// The offending prefix.
        prefix: String,
    },
}

/// What the ahead/behind counts were computed against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefBasis {
    /// Compared against the branch's upstream.
    Upstream,
    /// Compared against the default branch (fallback when no upstream).
    DefaultBranch,
}

/// ahead/behind vs upstream (or vs the default branch when no upstream).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackingFacet {
    /// Commits ahead.
    pub ahead: u32,
    /// Commits behind.
    pub behind: u32,
    /// Whether the upstream ref no longer exists on the remote.
    pub upstream_gone: bool,
    /// What the counts were compared against.
    pub compared_against: RefBasis,
}

/// Advisory suggested action for a branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recommendation {
    /// Keep — it is protected.
    KeepProtected,
    /// Safe to delete — merged.
    DeleteMerged,
    /// Review — unmerged.
    ReviewUnmerged,
    /// Archive — stale.
    ArchiveStale,
    /// No action suggested.
    NoAction,
}
