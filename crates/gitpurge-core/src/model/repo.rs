//! Repository identity & metadata (docs/03 §1).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::Branch;

/// Stable identity for a tracked repo. Derived, never user-typed.
///
/// Per CONVENTIONS §5: keyed by canonical remote URL + a hash of the local path, so
/// the same working copy and its mirror always resolve to one `RepoId`
/// (e.g. `"gh:MohamedGamil/git-purge#a1b2c3"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepoId(pub String);

/// A hint at the hosting provider, driving auth and future PR-metadata adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderHint {
    /// GitHub-hosted.
    GitHub,
    /// GitLab-hosted.
    GitLab,
    /// Bitbucket-hosted.
    Bitbucket,
    /// Plain git over ssh/https, no host API assumed.
    Generic,
    /// Provider could not be determined.
    Unknown,
}

/// The scheme of a parsed git URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrlScheme {
    /// `ssh://` or scp-like `git@host:owner/repo`.
    Ssh,
    /// `https://`.
    Https,
    /// `git://`.
    Git,
    /// `file://` or a local path.
    File,
}

/// Parsed, normalized git URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitUrl {
    /// URL scheme.
    pub scheme: UrlScheme,
    /// Host, if any (`None` for local `file` URLs).
    pub host: Option<String>,
    /// Owner / namespace, if the provider exposes one.
    pub owner: Option<String>,
    /// Repository name.
    pub name: String,
    /// Exactly what the user/config supplied.
    pub raw: String,
}

/// A repository Git Purge tracks. May be local-only, remote-only, or both.
///
/// At least one of `local_path` / `remote_url` is always present — enforced by the
/// constructor (added in P1), not the type system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    /// Stable derived identity.
    pub id: RepoId,
    /// User label; defaults to the repo dir / URL slug.
    pub display_name: String,
    /// Working copy or bare repo on disk.
    pub local_path: Option<PathBuf>,
    /// Canonical fetch/push URL.
    pub remote_url: Option<GitUrl>,
    /// Resolved default branch (via `DefaultBranchPolicy`), not assumed.
    pub default_branch: Option<Branch>,
    /// Provider hint.
    pub provider: ProviderHint,
    /// When the repo was first tracked.
    pub added_at: OffsetDateTime,
    /// When the repo was last scanned, if ever.
    pub last_scanned_at: Option<OffsetDateTime>,
}
