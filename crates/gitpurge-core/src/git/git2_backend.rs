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
        let git2_repo = git2::Repository::open(local_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open repository: {}", e)))?;
        let mut r = git2_repo
            .find_branch(&branch.0, git2::BranchType::Local)
            .map_err(|e| {
                crate::GitPurgeError::RefNotFound(format!("Local branch not found: {}", e))
            })?;
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
        let git2_repo = git2::Repository::open(local_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open repository: {}", e)))?;
        let mut git2_remote = git2_repo.find_remote(remote).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to find remote '{}': {}", remote, e))
        })?;

        let mut callbacks = get_remote_callbacks();

        // Fail push if remote rejects the reference update
        callbacks.push_update_reference(|refname, status| {
            if let Some(err) = status {
                Err(git2::Error::from_str(&format!(
                    "Remote rejected reference update for '{}': {}",
                    refname, err
                )))
            } else {
                Ok(())
            }
        });

        let refspec = format!(":refs/heads/{}", branch.0);
        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        git2_remote
            .push(&[&refspec], Some(&mut push_opts))
            .map_err(|e| {
                crate::GitPurgeError::Git(format!(
                    "Failed to push deletion of remote branch: {}",
                    e
                ))
            })?;

        // Explicitly clean up local remote-tracking reference to ensure consistency
        let tracking_ref_name = format!("refs/remotes/{}/{}", remote, branch.0);
        if let Ok(mut tracking_ref) = git2_repo.find_reference(&tracking_ref_name) {
            let _ = tracking_ref.delete();
        }

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
        let git2_repo = git2::Repository::open(local_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open repository: {}", e)))?;
        let oid = git2::Oid::from_str(&target.0)
            .map_err(|e| crate::GitPurgeError::Git(format!("Invalid target commit OID: {}", e)))?;
        git2_repo
            .reference(full_ref, oid, force, "git-purge create_ref")
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to create reference: {}", e)))?;
        Ok(())
    }

    fn fetch(&self, repo: &Repository, remote: &str) -> Result<()> {
        let local_path = repo.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;
        let git2_repo = git2::Repository::open(local_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open repository: {}", e)))?;
        let mut git2_remote = git2_repo.find_remote(remote).map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to find remote '{}': {}", remote, e))
        })?;
        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(get_remote_callbacks());
        git2_remote
            .fetch(&[] as &[&str], Some(&mut fetch_opts), None)
            .map_err(|e| {
                crate::GitPurgeError::Git(format!("Failed to fetch remote '{}': {}", remote, e))
            })?;
        Ok(())
    }

    fn fetch_all_prune(&self, repo: &Repository) -> Result<()> {
        let local_path = repo.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;
        let git2_repo = git2::Repository::open(local_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open repository: {}", e)))?;

        let remotes = git2_repo
            .remotes()
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to list remotes: {}", e)))?;

        for remote_name in remotes.iter().flatten() {
            let mut git2_remote = git2_repo.find_remote(remote_name).map_err(|e| {
                crate::GitPurgeError::Git(format!("Failed to find remote '{}': {}", remote_name, e))
            })?;

            let mut fetch_opts = git2::FetchOptions::new();
            fetch_opts.prune(git2::FetchPrune::On);
            fetch_opts.remote_callbacks(get_remote_callbacks());

            tracing::info!("Auto-fetching and pruning remote '{}'...", remote_name);
            git2_remote
                .fetch(&[] as &[&str], Some(&mut fetch_opts), None)
                .map_err(|e| {
                    crate::GitPurgeError::Git(format!(
                        "Failed to fetch remote '{}': {}",
                        remote_name, e
                    ))
                })?;
        }

        Ok(())
    }
}

fn get_remote_callbacks() -> git2::RemoteCallbacks<'static> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|url, username_from_url, allowed_types| {
        tracing::debug!(
            "CREDENTIALS CALLBACK CALLED! URL: {}, USERNAME: {:?}, ALLOWED: {:?}",
            url,
            username_from_url,
            allowed_types
        );
        let user = username_from_url.unwrap_or("git");

        if allowed_types.contains(git2::CredentialType::USERNAME) {
            tracing::debug!("RETURNING USERNAME CREDENTIAL: {}", user);
            return git2::Cred::username(user);
        }

        if allowed_types.contains(git2::CredentialType::SSH_KEY)
            || allowed_types.contains(git2::CredentialType::SSH_CUSTOM)
        {
            // 1. Try SSH agent first
            match git2::Cred::ssh_key_from_agent(user) {
                Ok(cred) => {
                    tracing::debug!("SUCCESS: LOADED KEY FROM SSH AGENT");
                    return Ok(cred);
                }
                Err(e) => {
                    tracing::debug!("SSH AGENT FAILED: {:?}", e);
                }
            }

            // 2. Try default SSH key files dynamically
            let ssh_dir = if let Some(bd) = directories::BaseDirs::new() {
                Some(bd.home_dir().join(".ssh"))
            } else if let Ok(home) = std::env::var("HOME") {
                Some(std::path::PathBuf::from(home).join(".ssh"))
            } else {
                None
            };

            if let Some(ssh_dir) = ssh_dir {
                let key_names = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];
                for name in &key_names {
                    let private_key = ssh_dir.join(name);
                    if private_key.exists() {
                        tracing::debug!("TRYING PRIVATE KEY FILE: {:?}", private_key);
                        match git2::Cred::ssh_key(user, None, &private_key, None) {
                            Ok(cred) => {
                                tracing::debug!("SUCCESS: LOADED KEY FILE {:?}", private_key);
                                return Ok(cred);
                            }
                            Err(e) => {
                                tracing::debug!(
                                    "FAILED TO LOAD KEY FILE {:?}: {:?}",
                                    private_key,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        // 3. Fallback to default
        tracing::debug!("FALLBACK TO DEFAULT CREDENTIALS");
        git2::Cred::default()
    });
    callbacks
}
