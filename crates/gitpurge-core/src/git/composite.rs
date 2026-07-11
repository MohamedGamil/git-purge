//! Composite git backend routing reads to gix and mutations to git2.

use crate::error::Result;
use crate::git::{FileDiffStat, Git2Backend, GitBackend, GixBackend};
use crate::model::{Branch, BranchName, BranchScope, Commit, Oid, Ref, RefSpec, Repository, Tag};

/// Composite backend routing read operations to `GixBackend` and mutate/fallback operations to `Git2Backend`.
#[derive(Debug, Clone, Default)]
pub struct CompositeGitBackend {
    /// The primary gix (gitoxide) read backend.
    pub gix: GixBackend,
    /// The fallback git2 (libgit2) write/network backend.
    pub git2: Git2Backend,
}

impl CompositeGitBackend {
    /// Create a new `CompositeGitBackend`.
    pub fn new() -> Self {
        Self {
            gix: GixBackend,
            git2: Git2Backend,
        }
    }
}

impl GitBackend for CompositeGitBackend {
    fn open_repo(&self, repo: &Repository) -> Result<()> {
        self.gix.open_repo(repo)
    }

    fn list_refs(&self, repo: &Repository) -> Result<Vec<Ref>> {
        self.gix.list_refs(repo)
    }

    fn list_branches(&self, repo: &Repository, scope: Option<BranchScope>) -> Result<Vec<Branch>> {
        self.gix.list_branches(repo, scope)
    }

    fn list_tags(&self, repo: &Repository) -> Result<Vec<Tag>> {
        self.gix.list_tags(repo)
    }

    fn resolve_ref(&self, repo: &Repository, spec: &RefSpec) -> Result<Commit> {
        self.gix.resolve_ref(repo, spec)
    }

    fn is_ancestor(&self, repo: &Repository, ancestor: &Oid, descendant: &Oid) -> Result<bool> {
        self.gix.is_ancestor(repo, ancestor, descendant)
    }

    fn merge_base(&self, repo: &Repository, a: &Oid, b: &Oid) -> Result<Option<Oid>> {
        self.gix.merge_base(repo, a, b)
    }

    fn read_tree(&self, repo: &Repository, at: &RefSpec) -> Result<Vec<String>> {
        self.gix.read_tree(repo, at)
    }

    fn read_blob(&self, repo: &Repository, at: &RefSpec, path: &str) -> Result<Vec<u8>> {
        self.gix.read_blob(repo, at, path)
    }

    fn diff_refs(&self, repo: &Repository, a: &RefSpec, b: &RefSpec) -> Result<Vec<FileDiffStat>> {
        self.gix.diff_refs(repo, a, b)
    }

    fn delete_local_branch(&self, repo: &Repository, branch: &BranchName) -> Result<()> {
        self.git2.delete_local_branch(repo, branch)
    }

    fn delete_remote_branch(
        &self,
        repo: &Repository,
        remote: &str,
        branch: &BranchName,
    ) -> Result<()> {
        self.git2.delete_remote_branch(repo, remote, branch)
    }

    fn create_ref(
        &self,
        repo: &Repository,
        full_ref: &str,
        target: &Oid,
        force: bool,
    ) -> Result<()> {
        self.git2.create_ref(repo, full_ref, target, force)
    }

    fn fetch(&self, repo: &Repository, remote: &str) -> Result<()> {
        self.git2.fetch(repo, remote)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit;

    #[test]
    fn test_composite_backend_routing() {
        let repo_fixture = testkit::merged_repo();
        let backend = CompositeGitBackend::new();

        let repo_model = Repository {
            id: crate::model::RepoId("test-composite".to_string()),
            display_name: "test-composite".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };

        // Reads should route to Gix and succeed
        assert!(backend.open_repo(&repo_model).is_ok());
        let refs = backend.list_refs(&repo_model).unwrap();
        assert!(refs.iter().any(|r| r.short == "main"));

        // Mutates route to Git2 (stubs return Ok in P1)
        assert!(backend
            .delete_local_branch(&repo_model, &BranchName("some-branch".to_string()))
            .is_ok());
    }
}
