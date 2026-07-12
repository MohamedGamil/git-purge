//! Bare-mirror manager (P2-T1, CONVENTIONS §6, docs/08 §2).

use crate::config::Config;
use crate::error::Result;
use crate::model::{RepoId, Repository};
use std::path::PathBuf;

/// Manages bare git mirrors used for backing up repositories.
pub struct BackupMirrorManager {
    backups_root: PathBuf,
}

impl BackupMirrorManager {
    /// Create a new mirror manager using the given configuration.
    pub fn new(config: &Config) -> Self {
        let backups_root = config
            .backups_root
            .clone()
            .unwrap_or_else(|| config.resolve_data_dir().join("backups"));
        Self { backups_root }
    }

    /// Resolve the on-disk path to the bare mirror for a repository.
    pub fn resolve_mirror_path(&self, repo_id: &RepoId) -> PathBuf {
        let sanitized_id: String = repo_id
            .0
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let target_path = self.backups_root.join(format!("{}.git", sanitized_id));
        if !target_path.exists() {
            // Fallback 1: If self.backups_root is ~/.gitpurge/backups, check if it was in ~/.gitpurge directly
            if let Some(parent) = self.backups_root.parent() {
                let other_path = parent.join(format!("{}.git", sanitized_id));
                if other_path.exists() {
                    return other_path;
                }
            }
            // Fallback 2: If self.backups_root is ~/.gitpurge, check if it was in ~/.gitpurge/backups
            let old_path = self
                .backups_root
                .join("backups")
                .join(format!("{}.git", sanitized_id));
            if old_path.exists() {
                return old_path;
            }
        }
        target_path
    }

    /// Open the bare mirror for a repository, initializing it if it does not exist.
    pub fn ensure_mirror(&self, repo: &Repository) -> Result<git2::Repository> {
        let mirror_path = self.resolve_mirror_path(&repo.id);

        let mirror_repo = if mirror_path.exists() {
            git2::Repository::open_bare(&mirror_path).map_err(|e| {
                crate::GitPurgeError::Git(format!("Failed to open bare mirror: {}", e))
            })?
        } else {
            if let Some(parent) = mirror_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            git2::Repository::init_bare(&mirror_path).map_err(|e| {
                crate::GitPurgeError::Git(format!("Failed to init bare mirror: {}", e))
            })?
        };

        Ok(mirror_repo)
    }

    /// Fetch all references (heads, tags, remotes) and objects from the source repository into the mirror.
    pub fn fetch_to_mirror(&self, repo: &Repository, mirror_repo: &git2::Repository) -> Result<()> {
        let source_path = repo.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;

        let source_path_str = source_path.to_string_lossy();
        let mut remote = mirror_repo
            .remote_anonymous(&source_path_str)
            .map_err(|e| {
                crate::GitPurgeError::Git(format!(
                    "Failed to create anonymous remote for mirror: {}",
                    e
                ))
            })?;

        let refspecs = &[
            "+refs/heads/*:refs/heads/*",
            "+refs/tags/*:refs/tags/*",
            "+refs/remotes/*:refs/remotes/*",
        ];

        let mut fetch_opts = git2::FetchOptions::new();
        remote
            .fetch(refspecs, Some(&mut fetch_opts), None)
            .map_err(|e| {
                crate::GitPurgeError::Git(format!("Failed to fetch objects to mirror: {}", e))
            })?;

        Ok(())
    }
}
