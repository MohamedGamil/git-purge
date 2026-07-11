//! Git objects: refs, branches, tags, commits, signatures (docs/03 §2).

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// A git object id (SHA-1 today, SHA-256-ready). Newtype over the hex string; no
/// field assumes 40 hex chars (docs/03 §10).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Oid(pub String);

/// A validated branch short name — never includes a `remote/` prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BranchName(pub String);

/// The git ref namespace a ref lives in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefKind {
    /// `refs/heads/*`.
    LocalBranch,
    /// `refs/remotes/<remote>/*`.
    RemoteBranch,
    /// `refs/tags/*`.
    Tag,
    /// `refs/notes/*` (read-only, never an Action target).
    Note,
    /// Anything else, incl. our own `refs/gitpurge/*` backup namespace.
    Other(String),
}

/// A fully-qualified reference plus the short name humans use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ref {
    /// e.g. `refs/remotes/origin/feature/x`.
    pub full: String,
    /// e.g. `origin/feature/x` or `feature/x`.
    pub short: String,
    /// Which namespace it lives in.
    pub kind: RefKind,
    /// The commit (or tag object) it points at.
    pub target: Oid,
}

/// Whether a branch is local or a remote-tracking ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchScope {
    /// `refs/heads/*`.
    Local,
    /// `refs/remotes/<remote>/*`.
    Remote,
}

/// Tracking relationship between a local branch and its remote counterpart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Upstream {
    /// e.g. `origin`.
    pub remote: String,
    /// The upstream branch name.
    pub ref_name: BranchName,
    /// Commits present locally but not upstream.
    pub ahead: u32,
    /// Commits present upstream but not locally.
    pub behind: u32,
}

/// A git identity + timestamp (author, committer, or tagger).
///
/// `email` is treated as PII: reports aggregate or redact it, and it never leaks into
/// logs (docs/03 §2, SAFE-07).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// Display name.
    pub name: String,
    /// Email address. PII — never written to shared reports raw.
    pub email: String,
    /// Timestamp of the action.
    pub when: OffsetDateTime,
}

/// Commit metadata as read from the object DB. No working-tree access needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    /// Full object id.
    pub oid: Oid,
    /// Abbreviated object id.
    pub short: String,
    /// Author identity.
    pub author: Signature,
    /// Committer identity.
    pub committer: Signature,
    /// Author date.
    pub author_date: OffsetDateTime,
    /// Commit date — the date staleness is measured against (docs/03 §4).
    pub commit_date: OffsetDateTime,
    /// First line of the message.
    pub subject: String,
    /// Parent commit ids.
    pub parents: Vec<Oid>,
}

/// A branch: the primary unit Git Purge classifies and acts on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    /// Short name, never including a remote prefix.
    pub name: BranchName,
    /// Local or remote.
    pub scope: BranchScope,
    /// `Some("origin")` when `scope == Remote`.
    pub remote: Option<String>,
    /// Canonical `refs/...` path.
    pub full_ref: String,
    /// Resolved tip commit metadata.
    pub tip: Commit,
    /// Tracking relationship, if any.
    pub upstream: Option<Upstream>,
    /// Whether this is the checked-out branch (never delete).
    pub is_head: bool,
}

/// A tag. First-class so it can be a *restore target* (restore-as-tag) but is
/// GUARDED against deletion by branch operations (CONVENTIONS §7.4, SAFE-03).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// Tag name.
    pub name: String,
    /// Object the tag points at.
    pub target: Oid,
    /// Whether the tag is annotated.
    pub annotated: bool,
    /// Tagger identity, for annotated tags.
    pub tagger: Option<Signature>,
    /// Tag message, for annotated tags.
    pub message: Option<String>,
}
