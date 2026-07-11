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
impl GitUrl {
    /// Parse a raw git URL or path.
    pub fn parse(raw: &str) -> crate::error::Result<Self> {
        let raw_trimmed = raw.trim();
        if raw_trimmed.starts_with("https://") || raw_trimmed.starts_with("http://") {
            let parsed = raw_trimmed
                .strip_prefix("https://")
                .unwrap_or_else(|| raw_trimmed.strip_prefix("http://").unwrap());
            let parts: Vec<&str> = parsed.split('/').collect();
            let host = parts.first().map(|s| s.to_string());
            let name_with_git = parts.last().unwrap_or(&"").to_string();
            let name = name_with_git
                .strip_suffix(".git")
                .unwrap_or(&name_with_git)
                .to_string();
            let owner = if parts.len() > 2 {
                Some(parts[1..parts.len() - 1].join("/"))
            } else {
                None
            };
            Ok(Self {
                scheme: UrlScheme::Https,
                host,
                owner,
                name,
                raw: raw.to_string(),
            })
        } else if raw_trimmed.starts_with("ssh://") {
            let parsed = raw_trimmed.strip_prefix("ssh://").unwrap();
            let parts: Vec<&str> = parsed.split('/').collect();
            let host_parts: Vec<&str> = parts[0].split('@').collect();
            let host = host_parts.last().map(|s| s.to_string());
            let name_with_git = parts.last().unwrap_or(&"").to_string();
            let name = name_with_git
                .strip_suffix(".git")
                .unwrap_or(&name_with_git)
                .to_string();
            let owner = if parts.len() > 2 {
                Some(parts[1..parts.len() - 1].join("/"))
            } else {
                None
            };
            Ok(Self {
                scheme: UrlScheme::Ssh,
                host,
                owner,
                name,
                raw: raw.to_string(),
            })
        } else if raw_trimmed.contains('@') && raw_trimmed.contains(':') {
            let parts: Vec<&str> = raw_trimmed.split(':').collect();
            let host_part = parts[0];
            let host_parts: Vec<&str> = host_part.split('@').collect();
            let host = host_parts.last().map(|s| s.to_string());
            let path_part = parts[1];
            let path_parts: Vec<&str> = path_part.split('/').collect();
            let name_with_git = path_parts.last().unwrap_or(&"").to_string();
            let name = name_with_git
                .strip_suffix(".git")
                .unwrap_or(&name_with_git)
                .to_string();
            let owner = if path_parts.len() > 1 {
                Some(path_parts[0..path_parts.len() - 1].join("/"))
            } else {
                None
            };
            Ok(Self {
                scheme: UrlScheme::Ssh,
                host,
                owner,
                name,
                raw: raw.to_string(),
            })
        } else {
            let path = std::path::Path::new(raw_trimmed);
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "local".to_string());
            Ok(Self {
                scheme: UrlScheme::File,
                host: None,
                owner: None,
                name,
                raw: raw.to_string(),
            })
        }
    }
}

impl ProviderHint {
    /// Guess provider hint from host string.
    pub fn from_host(host: Option<&str>) -> Self {
        match host {
            Some(h) => {
                let h_lower = h.to_lowercase();
                if h_lower.contains("github.com") {
                    Self::GitHub
                } else if h_lower.contains("gitlab.com") {
                    Self::GitLab
                } else if h_lower.contains("bitbucket.org") {
                    Self::Bitbucket
                } else {
                    Self::Generic
                }
            }
            None => Self::Unknown,
        }
    }
}

impl Repository {
    /// Build a new Repository from a local path.
    pub fn new_local(path: PathBuf) -> crate::error::Result<Self> {
        let canonical_path = path.canonicalize().map_err(|e| {
            crate::GitPurgeError::RepoNotFound(format!(
                "Failed to canonicalize path {:?}: {}",
                path, e
            ))
        })?;

        let mut remote_url = None;
        let mut provider = ProviderHint::Unknown;
        if let Ok(repo) = git2::Repository::open(&canonical_path) {
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url_str) = remote.url() {
                    if let Ok(git_url) = GitUrl::parse(url_str) {
                        provider = ProviderHint::from_host(git_url.host.as_deref());
                        remote_url = Some(git_url);
                    }
                }
            }
        }

        let display_name = canonical_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "local".to_string());

        // Derive stable RepoId using a hash of the local path
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        canonical_path.hash(&mut hasher);
        let path_hash = format!("{:x}", hasher.finish());
        let short_hash = &path_hash[..std::cmp::min(6, path_hash.len())];

        let id_str = if let Some(ref remote) = remote_url {
            let provider_prefix = match provider {
                ProviderHint::GitHub => "gh",
                ProviderHint::GitLab => "gl",
                ProviderHint::Bitbucket => "bb",
                _ => "generic",
            };
            let owner_part = remote.owner.as_deref().unwrap_or("unknown");
            format!(
                "{}:{}/{}#{}",
                provider_prefix, owner_part, remote.name, short_hash
            )
        } else {
            format!("local:{}#{}", display_name, short_hash)
        };

        Ok(Self {
            id: RepoId(id_str),
            display_name,
            local_path: Some(canonical_path),
            remote_url,
            default_branch: None,
            provider,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        })
    }

    /// Build a new Repository from a remote URL.
    pub fn new_remote(url: GitUrl) -> crate::error::Result<Self> {
        let provider = ProviderHint::from_host(url.host.as_deref());
        let provider_prefix = match provider {
            ProviderHint::GitHub => "gh",
            ProviderHint::GitLab => "gl",
            ProviderHint::Bitbucket => "bb",
            _ => "generic",
        };
        let owner_part = url.owner.as_deref().unwrap_or("unknown");
        let id_str = format!("{}:{}/{}", provider_prefix, owner_part, url.name);

        Ok(Self {
            id: RepoId(id_str),
            display_name: url.name.clone(),
            local_path: None,
            remote_url: Some(url),
            default_branch: None,
            provider,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        })
    }
}
