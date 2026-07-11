//! Git backend port — the central abstraction for all git operations (docs/02 §3).
//!
//! `GitBackend` is the trait that decouples `gitpurge-core` from any specific git
//! library. Production adapters (gix, git2, shell) implement this trait; tests use
//! `FakeGitBackend`.
//!
//! ## Design (CONVENTIONS §4)
//! - **gix (gitoxide)** — primary, for reads.
//! - **git2 (libgit2)** — fallback for operations gix doesn't cover.
//! - **System `git` shell-out** — last-resort adapter.

use crate::error::Result;
use crate::model::{Branch, BranchName, BranchScope, Commit, Oid, Ref, RefSpec, Repository, Tag};

/// A diff stat for a single file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileDiffStat {
    /// Path of the file.
    pub path: String,
    /// Lines added.
    pub additions: u32,
    /// Lines removed.
    pub deletions: u32,
}

/// The central git abstraction. Every git operation goes through this trait.
///
/// Implementations must be `Send + Sync` so `Engine` can be shared across threads.
///
/// # Conventions
/// - Methods that read return domain types; methods that mutate return `Result<()>`
///   or a confirmation type.
/// - No method ever touches credentials directly — auth is handled by `SecretStore`
///   and credential callbacks are wired externally.
pub trait GitBackend: Send + Sync + std::fmt::Debug {
    /// Open/validate a repository at the given path or URL.
    fn open_repo(&self, repo: &Repository) -> Result<()>;

    /// Enumerate all refs (branches + tags) in the repository.
    fn list_refs(&self, repo: &Repository) -> Result<Vec<Ref>>;

    /// List branches, optionally filtered by scope.
    fn list_branches(
        &self,
        repo: &Repository,
        scope: Option<BranchScope>,
    ) -> Result<Vec<Branch>>;

    /// List tags.
    fn list_tags(&self, repo: &Repository) -> Result<Vec<Tag>>;

    /// Resolve a ref spec to a commit.
    fn resolve_ref(&self, repo: &Repository, spec: &RefSpec) -> Result<Commit>;

    /// Check if `ancestor` is an ancestor of `descendant` (merge-base --is-ancestor).
    fn is_ancestor(&self, repo: &Repository, ancestor: &Oid, descendant: &Oid) -> Result<bool>;

    /// Compute the merge-base of two commits.
    fn merge_base(&self, repo: &Repository, a: &Oid, b: &Oid) -> Result<Option<Oid>>;

    /// Read the file tree at a given ref/commit.
    fn read_tree(&self, repo: &Repository, at: &RefSpec) -> Result<Vec<String>>;

    /// Read a single blob (file content) at a ref + path.
    fn read_blob(&self, repo: &Repository, at: &RefSpec, path: &str) -> Result<Vec<u8>>;

    /// Compute diff stats between two refs.
    fn diff_refs(
        &self,
        repo: &Repository,
        a: &RefSpec,
        b: &RefSpec,
    ) -> Result<Vec<FileDiffStat>>;

    /// Delete a local branch.
    fn delete_local_branch(&self, repo: &Repository, branch: &BranchName) -> Result<()>;

    /// Delete a remote branch (push --delete).
    fn delete_remote_branch(
        &self,
        repo: &Repository,
        remote: &str,
        branch: &BranchName,
    ) -> Result<()>;

    /// Create a ref pointing at the given OID.
    fn create_ref(
        &self,
        repo: &Repository,
        full_ref: &str,
        target: &Oid,
        force: bool,
    ) -> Result<()>;

    /// Fetch from a remote.
    fn fetch(&self, repo: &Repository, remote: &str) -> Result<()>;
}

/// In-memory fake for tests. Proves dependency-inversion (P0-T4).
#[derive(Debug, Default)]
pub struct FakeGitBackend {
    // TODO(P0-T4): add fields to store canned responses for each method.
}

impl GitBackend for FakeGitBackend {
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
        Err(crate::GitPurgeError::RefNotFound("fake".into()))
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
