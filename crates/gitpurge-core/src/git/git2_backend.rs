//! Fallback git2 (libgit2) backend (CONVENTIONS §4).

use crate::error::Result;
use crate::git::{FileDiffStat, GitBackend};
use crate::model::{Branch, BranchName, BranchScope, Commit, Oid, Ref, RefSpec, Repository, Tag};

/// Git2 backend implementation of `GitBackend`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Git2Backend;

impl GitBackend for Git2Backend {
    fn open_repo(&self, _repo: &Repository) -> Result<()> {
        Ok(())
    }

    fn list_refs(&self, _repo: &Repository) -> Result<Vec<Ref>> {
        Ok(Vec::new())
    }

    fn list_branches(
        &self,
        _repo: &Repository,
        _scope: Option<BranchScope>,
    ) -> Result<Vec<Branch>> {
        Ok(Vec::new())
    }

    fn list_tags(&self, _repo: &Repository) -> Result<Vec<Tag>> {
        Ok(Vec::new())
    }

    fn resolve_ref(&self, _repo: &Repository, _spec: &RefSpec) -> Result<Commit> {
        Err(crate::GitPurgeError::RefNotFound(
            "Not implemented in Git2Backend stub".to_string(),
        ))
    }

    fn is_ancestor(&self, _repo: &Repository, _ancestor: &Oid, _descendant: &Oid) -> Result<bool> {
        Ok(false)
    }

    fn merge_base(&self, _repo: &Repository, _a: &Oid, _b: &Oid) -> Result<Option<Oid>> {
        Ok(None)
    }

    fn read_tree(&self, _repo: &Repository, _at: &RefSpec) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn read_blob(&self, _repo: &Repository, _at: &RefSpec, _path: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn diff_refs(
        &self,
        _repo: &Repository,
        _a: &RefSpec,
        _b: &RefSpec,
    ) -> Result<Vec<FileDiffStat>> {
        Ok(Vec::new())
    }

    fn delete_local_branch(&self, repo: &Repository, branch: &BranchName) -> Result<()> {
        let local_path = repo.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;
        let git2_repo = git2::Repository::open(local_path).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to open repository: {}", e))
        })?;
        let mut r = git2_repo
            .find_branch(&branch.0, git2::BranchType::Local)
            .map_err(|e| crate::GitPurgeError::RefNotFound(format!("Local branch not found: {}", e)))?;
        r.delete().map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to delete local branch: {}", e))
        })?;
        Ok(())
    }

    fn delete_remote_branch(
        &self,
        repo: &Repository,
        remote: &str,
        branch: &BranchName,
    ) -> Result<()> {
        let local_path = repo.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;
        let git2_repo = git2::Repository::open(local_path).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to open repository: {}", e))
        })?;
        let mut git2_remote = git2_repo.find_remote(remote).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to find remote '{}': {}", remote, e))
        })?;
        let refspec = format!(":refs/heads/{}", branch.0);
        let mut push_opts = git2::PushOptions::new();
        git2_remote.push(&[&refspec], Some(&mut push_opts)).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to push deletion of remote branch: {}", e))
        })?;
        Ok(())
    }

    fn create_ref(
        &self,
        repo: &Repository,
        full_ref: &str,
        target: &Oid,
        force: bool,
    ) -> Result<()> {
        let local_path = repo.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;
        let git2_repo = git2::Repository::open(local_path).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to open repository: {}", e))
        })?;
        let oid = git2::Oid::from_str(&target.0).map_err(|e| {
            crate::GitPurgeError::Git(format!("Invalid target commit OID: {}", e))
        })?;
        git2_repo.reference(full_ref, oid, force, "git-purge create_ref").map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to create reference: {}", e))
        })?;
        Ok(())
    }

    fn fetch(&self, repo: &Repository, remote: &str) -> Result<()> {
        let local_path = repo.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;
        let git2_repo = git2::Repository::open(local_path).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to open repository: {}", e))
        })?;
        let mut git2_remote = git2_repo.find_remote(remote).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to find remote '{}': {}", remote, e))
        })?;
        let mut fetch_opts = git2::FetchOptions::new();
        git2_remote
            .fetch(&[] as &[&str], Some(&mut fetch_opts), None)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to fetch remote '{}': {}", remote, e)))?;
        Ok(())
    }
}
