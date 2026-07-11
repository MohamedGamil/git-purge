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

    fn delete_local_branch(&self, _repo: &Repository, _branch: &BranchName) -> Result<()> {
        Ok(())
    }

    fn delete_remote_branch(
        &self,
        _repo: &Repository,
        _remote: &str,
        _branch: &BranchName,
    ) -> Result<()> {
        Ok(())
    }

    fn create_ref(
        &self,
        _repo: &Repository,
        _full_ref: &str,
        _target: &Oid,
        _force: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn fetch(&self, _repo: &Repository, _remote: &str) -> Result<()> {
        Ok(())
    }
}
