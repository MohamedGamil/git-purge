//! Git read operations (P10-T2).

use crate::error::Result;
use crate::model::{RefSpec, RepoId};
use std::path::Path;

impl super::Engine {
    /// Fetch from remote (default 'origin') for a repository.
    pub fn fetch(&self, repo_id: &RepoId) -> Result<()> {
        let repo = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo_id).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!(
                    "Repository not registered: {:?}",
                    repo_id
                ))
            })?
        };
        self.git.fetch(&repo, "origin")
    }

    /// Diff two refs.
    pub fn diff(&self, repo: &RepoId, a: &RefSpec, b: &RefSpec) -> Result<crate::diff::DiffResult> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        let diffs = self.git.diff_refs(&repo_model, a, b)?;
        let mut entries = Vec::new();
        let mut insertions = 0;
        let mut deletions = 0;

        for d in diffs {
            let kind = if d.additions > 0 && d.deletions > 0 {
                crate::diff::DiffKind::Modified
            } else if d.additions > 0 {
                crate::diff::DiffKind::Added
            } else {
                crate::diff::DiffKind::Deleted
            };

            insertions += d.additions;
            deletions += d.deletions;

            entries.push(crate::diff::DiffEntry {
                path: d.path,
                kind,
                additions: d.additions,
                deletions: d.deletions,
            });
        }

        let files_changed = entries.len();

        Ok(crate::diff::DiffResult {
            from: a.clone(),
            to: b.clone(),
            entries,
            files_changed,
            insertions,
            deletions,
        })
    }

    /// View the tree (or a single path) at a ref/commit.
    pub fn show_tree(
        &self,
        repo: &RepoId,
        at: &RefSpec,
        path: Option<&Path>,
    ) -> Result<crate::diff::TreeView> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };

        if let Some(p) = path {
            let blob_data = self
                .git
                .read_blob(&repo_model, at, p.to_str().unwrap_or(""))?;
            let entry = crate::diff::TreeEntry {
                path: p.to_string_lossy().to_string(),
                is_dir: false,
                size: blob_data.len() as u64,
                oid: crate::model::Oid("fake-oid".to_string()),
            };
            Ok(crate::diff::TreeView {
                at: at.clone(),
                entries: vec![entry],
            })
        } else {
            let files = self.git.read_tree(&repo_model, at)?;
            let mut entries = Vec::new();
            for file in files {
                entries.push(crate::diff::TreeEntry {
                    path: file,
                    is_dir: false,
                    size: 0,
                    oid: crate::model::Oid("fake-oid".to_string()),
                });
            }
            Ok(crate::diff::TreeView {
                at: at.clone(),
                entries,
            })
        }
    }

    /// Read the raw content of a file at a ref/commit.
    pub fn show_file(&self, repo: &RepoId, at: &RefSpec, path: &Path) -> Result<Vec<u8>> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", repo))
            })?
        };
        self.git
            .read_blob(&repo_model, at, path.to_str().unwrap_or(""))
    }
}
