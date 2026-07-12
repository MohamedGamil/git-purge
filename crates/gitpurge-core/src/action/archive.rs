//! Programmatic branch archiving (P3-T2, CONVENTIONS §8, docs/04 §3.1).

use crate::config::Config;
use crate::error::{GitPurgeError, Result};
use crate::model::{BranchName, Repository};

/// Strategy to use when merging unmerged branches into the legacy branch.
pub enum ArchiveStrategy {
    /// Ours strategy: keep target branch content, discard incoming files, keep branch history.
    Ours,
    /// Theirs strategy: merge incoming files, prefer incoming on conflicts.
    Theirs,
}

/// Archive a list of stale unmerged branches into a legacy branch.
pub fn archive_branches(
    _config: &Config,
    repo_model: &Repository,
    branches_to_archive: &[BranchName],
    target_branch: &str,
    strategy: ArchiveStrategy,
    push_after: bool,
) -> Result<()> {
    if branches_to_archive.is_empty() {
        return Ok(());
    }

    let local_path = repo_model.local_path.as_ref().ok_or_else(|| {
        GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
    })?;

    let repo = git2::Repository::open(local_path)
        .map_err(|e| GitPurgeError::Git(format!("Failed to open repository: {}", e)))?;

    // 1. Resolve or create target legacy branch
    let mut target_commit = match repo.find_branch(target_branch, git2::BranchType::Local) {
        Ok(branch) => branch.into_reference().peel_to_commit().map_err(|e| {
            GitPurgeError::Git(format!("Failed to resolve target branch commit: {}", e))
        })?,
        Err(_) => {
            // Target does not exist, create it pointing to default branch (e.g. main/master)
            // Let's resolve the HEAD or default branch
            let default_ref_name = repo_model
                .default_branch
                .as_ref()
                .map(|b| b.full_ref.clone())
                .unwrap_or_else(|| "refs/heads/main".to_string());
            let default_ref = repo
                .find_reference(&default_ref_name)
                .or_else(|_| repo.find_reference("refs/heads/master"))
                .or_else(|_| repo.head())
                .map_err(|e| {
                    GitPurgeError::Git(format!(
                        "Failed to find default branch to initialize legacy archive: {}",
                        e
                    ))
                })?;
            let commit = default_ref.peel_to_commit().map_err(|e| {
                GitPurgeError::Git(format!("Failed to peel default reference to commit: {}", e))
            })?;
            repo.branch(target_branch, &commit, false).map_err(|e| {
                GitPurgeError::Git(format!(
                    "Failed to create legacy branch '{}': {}",
                    target_branch, e
                ))
            })?;
            commit
        }
    };

    let sig = repo
        .signature()
        .unwrap_or_else(|_| git2::Signature::now("Git Purge", "gitpurge@localhost").unwrap());

    // 2. Perform merges for each source branch
    for source in branches_to_archive {
        let res = (|| -> Result<()> {
            let source_branch = repo
                .find_branch(&source.0, git2::BranchType::Local)
                .or_else(|_| {
                    repo.find_branch(&format!("origin/{}", source.0), git2::BranchType::Remote)
                })
                .map_err(|e| {
                    GitPurgeError::RefNotFound(format!(
                        "Archive source branch '{}' not found: {}",
                        source.0, e
                    ))
                })?;
            let source_commit = source_branch
                .into_reference()
                .peel_to_commit()
                .map_err(|e| {
                    GitPurgeError::Git(format!("Failed to peel source branch to commit: {}", e))
                })?;

            match strategy {
                ArchiveStrategy::Ours => {
                    // Ours: merge commit keeping target branch's tree (discard source diff, keep history)
                    let tree = target_commit.tree().map_err(|e| {
                        GitPurgeError::Git(format!("Failed to get target tree: {}", e))
                    })?;
                    let merge_oid = repo
                        .commit(
                            Some(&format!("refs/heads/{}", target_branch)),
                            &sig,
                            &sig,
                            &format!("Archive merge branch '{}' (ours)", source.0),
                            &tree,
                            &[&target_commit, &source_commit],
                        )
                        .map_err(|e| {
                            GitPurgeError::Git(format!("Failed to create ours merge commit: {}", e))
                        })?;
                    target_commit = repo.find_commit(merge_oid).unwrap();
                }
                ArchiveStrategy::Theirs => {
                    // Theirs: merge prefers source branch's files on conflict
                    let mut index = repo
                        .merge_commits(&target_commit, &source_commit, None)
                        .map_err(|e| {
                            GitPurgeError::Git(format!("Failed to merge commits: {}", e))
                        })?;

                    if index.has_conflicts() {
                        let mut their_entries = Vec::new();
                        let conflicts = index.conflicts().map_err(|e| {
                            GitPurgeError::Git(format!("Failed to retrieve conflicts: {}", e))
                        })?;
                        for conflict in conflicts {
                            let conflict = conflict.map_err(|e| {
                                GitPurgeError::Git(format!("Conflict error: {}", e))
                            })?;
                            if let Some(their_entry) = conflict.their {
                                their_entries.push(their_entry);
                            }
                        }
                        for their_entry in their_entries {
                            index.add(&their_entry).map_err(|e| {
                                GitPurgeError::Git(format!(
                                    "Failed to add their conflict entry: {}",
                                    e
                                ))
                            })?;
                        }
                    }

                    let tree_id = index.write_tree_to(&repo).map_err(|e| {
                        GitPurgeError::Git(format!("Failed to write merged index to tree: {}", e))
                    })?;
                    let tree = repo.find_tree(tree_id).map_err(|e| {
                        GitPurgeError::Git(format!("Failed to find merged tree: {}", e))
                    })?;

                    let merge_oid = repo
                        .commit(
                            Some(&format!("refs/heads/{}", target_branch)),
                            &sig,
                            &sig,
                            &format!("Archive merge branch '{}' (theirs)", source.0),
                            &tree,
                            &[&target_commit, &source_commit],
                        )
                        .map_err(|e| {
                            GitPurgeError::Git(format!(
                                "Failed to create theirs merge commit: {}",
                                e
                            ))
                        })?;
                    target_commit = repo.find_commit(merge_oid).unwrap();
                }
            }
            Ok(())
        })();

        match &res {
            Ok(()) => crate::log_operation("ARCHIVE", &source.0, "local/remote", "SUCCESS"),
            Err(e) => crate::log_operation(
                "ARCHIVE",
                &source.0,
                "local/remote",
                &format!("FAILED: {}", e),
            ),
        }
        res?;
    }

    // 3. Push to remote if requested
    if push_after {
        if let Ok(mut remote) = repo.find_remote("origin") {
            let refspec = format!("refs/heads/{}", target_branch);
            let mut push_opts = git2::PushOptions::new();
            remote
                .push(&[&refspec], Some(&mut push_opts))
                .map_err(|e| {
                    GitPurgeError::Git(format!("Failed to push legacy branch to origin: {}", e))
                })?;
        }
    }

    Ok(())
}
