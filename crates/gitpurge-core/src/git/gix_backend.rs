//! Production gix (gitoxide) backend (CONVENTIONS §4).

use crate::error::Result;
use crate::git::{FileDiffStat, GitBackend};
use crate::model::{
    Branch, BranchName, BranchScope, Commit, Oid, Ref, RefSpec, Repository, Signature, Tag,
};

/// Gix backend implementation of `GitBackend`.
#[derive(Debug, Clone, Copy, Default)]
pub struct GixBackend;

impl GitBackend for GixBackend {
    fn open_repo(&self, repo: &Repository) -> Result<()> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        Ok(())
    }

    fn list_refs(&self, repo: &Repository) -> Result<Vec<Ref>> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let refs = gix_repo
            .references()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let mut result = Vec::new();
        for r in refs
            .all()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
        {
            let mut r = r.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            let full_name = r.name().as_bstr().to_string();
            let short_name = r.name().shorten().to_string();

            let peeled_id = match r.peel_to_id_in_place() {
                Ok(id) => id.detach(),
                Err(_) => continue,
            };

            let kind = if full_name.starts_with("refs/heads/") {
                crate::model::RefKind::LocalBranch
            } else if full_name.starts_with("refs/remotes/") {
                crate::model::RefKind::RemoteBranch
            } else if full_name.starts_with("refs/tags/") {
                crate::model::RefKind::Tag
            } else {
                crate::model::RefKind::Other(full_name.clone())
            };

            result.push(Ref {
                full: full_name,
                short: short_name,
                kind,
                target: Oid(peeled_id.to_string()),
            });
        }
        Ok(result)
    }

    fn list_branches(&self, repo: &Repository, scope: Option<BranchScope>) -> Result<Vec<Branch>> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let head = gix_repo
            .head()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let head_ref_name = head.referent_name().map(|n| n.as_bstr().to_string());

        let refs = gix_repo
            .references()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let mut branches = Vec::new();

        for r in refs
            .all()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
        {
            let mut r = r.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            let full_name = r.name().as_bstr().to_string();

            let (branch_scope, short_name, remote) = if full_name.starts_with("refs/heads/") {
                (BranchScope::Local, r.name().shorten().to_string(), None)
            } else if let Some(stripped) = full_name.strip_prefix("refs/remotes/") {
                let parts: Vec<&str> = stripped.splitn(2, '/').collect();
                if parts.len() == 2 {
                    (
                        BranchScope::Remote,
                        parts[1].to_string(),
                        Some(parts[0].to_string()),
                    )
                } else {
                    continue;
                }
            } else {
                continue;
            };

            if let Some(s) = scope {
                if s != branch_scope {
                    continue;
                }
            }

            let peeled_id = match r.peel_to_id_in_place() {
                Ok(id) => id.detach(),
                Err(_) => continue,
            };

            let is_head = Some(&full_name) == head_ref_name.as_ref();

            let tip_commit = read_commit_internal(&gix_repo, peeled_id)?;

            branches.push(Branch {
                name: BranchName(short_name),
                scope: branch_scope,
                remote,
                full_ref: full_name,
                tip: tip_commit,
                upstream: None,
                is_head,
            });
        }
        Ok(branches)
    }

    fn list_tags(&self, repo: &Repository) -> Result<Vec<Tag>> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let refs = gix_repo
            .references()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let mut tags = Vec::new();

        for r in refs
            .all()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
        {
            let mut r = r.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            let full_name = r.name().as_bstr().to_string();
            if !full_name.starts_with("refs/tags/") {
                continue;
            }

            let short_name = r.name().shorten().to_string();
            let peeled_id = match r.peel_to_id_in_place() {
                Ok(id) => id.detach(),
                Err(_) => continue,
            };

            tags.push(Tag {
                name: short_name,
                target: Oid(peeled_id.to_string()),
                annotated: false,
                tagger: None,
                message: None,
            });
        }
        Ok(tags)
    }

    fn resolve_ref(&self, repo: &Repository, spec: &RefSpec) -> Result<Commit> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let oid = match spec {
            RefSpec::Branch(name) => {
                let r = gix_repo
                    .find_reference(format!("refs/heads/{}", name.0).as_str())
                    .or_else(|_| gix_repo.find_reference(name.0.as_str()))
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
            RefSpec::Tag(name) => {
                let r = gix_repo
                    .find_reference(format!("refs/tags/{}", name).as_str())
                    .or_else(|_| gix_repo.find_reference(name.as_str()))
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
            RefSpec::Oid(oid) => gix::hash::ObjectId::from_hex(oid.0.as_bytes())
                .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?,
            RefSpec::Symbolic(name) => {
                let r = gix_repo
                    .find_reference(name.as_str())
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
        };

        read_commit_internal(&gix_repo, oid)
    }

    fn is_ancestor(&self, repo: &Repository, ancestor: &Oid, descendant: &Oid) -> Result<bool> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let anc_id = gix::hash::ObjectId::from_hex(ancestor.0.as_bytes())
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let desc_id = gix::hash::ObjectId::from_hex(descendant.0.as_bytes())
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let rev_walk = gix_repo.rev_walk([desc_id]);
        for commit_info in rev_walk
            .all()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
        {
            let info = commit_info.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            if info.id == anc_id {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn merge_base(&self, repo: &Repository, a: &Oid, b: &Oid) -> Result<Option<Oid>> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let a_id = gix::hash::ObjectId::from_hex(a.0.as_bytes())
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let b_id = gix::hash::ObjectId::from_hex(b.0.as_bytes())
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        // Walk ancestors of a and collect them
        let mut ancestors_a = std::collections::HashSet::new();
        let walk_a = gix_repo.rev_walk([a_id]);
        for commit_info in walk_a
            .all()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
        {
            let info = commit_info.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            ancestors_a.insert(info.id);
        }

        // Walk ancestors of b and find the first one in ancestors_a
        let walk_b = gix_repo.rev_walk([b_id]);
        for commit_info in walk_b
            .all()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
        {
            let info = commit_info.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            if ancestors_a.contains(&info.id) {
                return Ok(Some(Oid(info.id.to_string())));
            }
        }

        Ok(None)
    }

    fn read_tree(&self, repo: &Repository, at: &RefSpec) -> Result<Vec<String>> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let oid = match at {
            RefSpec::Branch(name) => {
                let r = gix_repo
                    .find_reference(format!("refs/heads/{}", name.0).as_str())
                    .or_else(|_| gix_repo.find_reference(name.0.as_str()))
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
            RefSpec::Tag(name) => {
                let r = gix_repo
                    .find_reference(format!("refs/tags/{}", name).as_str())
                    .or_else(|_| gix_repo.find_reference(name.as_str()))
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
            RefSpec::Oid(oid) => gix::hash::ObjectId::from_hex(oid.0.as_bytes())
                .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?,
            RefSpec::Symbolic(name) => {
                let r = gix_repo
                    .find_reference(name.as_str())
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
        };

        let commit = gix_repo
            .find_object(oid)
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
            .try_into_commit()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let tree = commit
            .tree()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let mut paths = Vec::new();
        walk_tree_recursive(&gix_repo, tree, "", &mut paths)?;
        Ok(paths)
    }

    fn read_blob(&self, repo: &Repository, at: &RefSpec, path: &str) -> Result<Vec<u8>> {
        let repo_path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo =
            gix::open(repo_path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let oid = match at {
            RefSpec::Branch(name) => {
                let r = gix_repo
                    .find_reference(format!("refs/heads/{}", name.0).as_str())
                    .or_else(|_| gix_repo.find_reference(name.0.as_str()))
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
            RefSpec::Tag(name) => {
                let r = gix_repo
                    .find_reference(format!("refs/tags/{}", name).as_str())
                    .or_else(|_| gix_repo.find_reference(name.as_str()))
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
            RefSpec::Oid(oid) => gix::hash::ObjectId::from_hex(oid.0.as_bytes())
                .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?,
            RefSpec::Symbolic(name) => {
                let r = gix_repo
                    .find_reference(name.as_str())
                    .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                let mut peeled = r;
                peeled
                    .peel_to_id_in_place()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .detach()
            }
        };

        let commit = gix_repo
            .find_object(oid)
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
            .try_into_commit()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let tree = commit
            .tree()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let mut buf = Vec::new();
        let entry = tree
            .lookup_entry_by_path(path, &mut buf)
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
            .ok_or_else(|| crate::GitPurgeError::RefNotFound(format!("File {} not found", path)))?;

        let obj = entry
            .object()
            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        Ok(obj.data.clone())
    }

    fn diff_refs(&self, repo: &Repository, a: &RefSpec, b: &RefSpec) -> Result<Vec<FileDiffStat>> {
        let path = repo
            .local_path
            .as_ref()
            .ok_or_else(|| crate::GitPurgeError::RepoNotFound("Local path not set".to_string()))?;
        let gix_repo = gix::open(path).map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

        let get_tree_entries =
            |spec: &RefSpec| -> Result<std::collections::HashMap<String, gix::hash::ObjectId>> {
                let oid = match spec {
                    RefSpec::Branch(name) => {
                        let r = gix_repo
                            .find_reference(format!("refs/heads/{}", name.0).as_str())
                            .or_else(|_| gix_repo.find_reference(name.0.as_str()))
                            .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                        let mut peeled = r;
                        peeled
                            .peel_to_id_in_place()
                            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                            .detach()
                    }
                    RefSpec::Tag(name) => {
                        let r = gix_repo
                            .find_reference(format!("refs/tags/{}", name).as_str())
                            .or_else(|_| gix_repo.find_reference(name.as_str()))
                            .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                        let mut peeled = r;
                        peeled
                            .peel_to_id_in_place()
                            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                            .detach()
                    }
                    RefSpec::Oid(oid) => gix::hash::ObjectId::from_hex(oid.0.as_bytes())
                        .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?,
                    RefSpec::Symbolic(name) => {
                        let r = gix_repo
                            .find_reference(name.as_str())
                            .map_err(|e| crate::GitPurgeError::RefNotFound(e.to_string()))?;
                        let mut peeled = r;
                        peeled
                            .peel_to_id_in_place()
                            .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                            .detach()
                    }
                };

                let commit = gix_repo
                    .find_object(oid)
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?
                    .try_into_commit()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

                let tree = commit
                    .tree()
                    .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

                let mut entries = std::collections::HashMap::new();

                fn walk(
                    r_repo: &gix::Repository,
                    t: gix::Tree<'_>,
                    prefix: &str,
                    out: &mut std::collections::HashMap<String, gix::hash::ObjectId>,
                ) -> Result<()> {
                    for entry in t.iter() {
                        let entry = entry.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
                        let name = entry.filename().to_string();
                        let path = if prefix.is_empty() {
                            name
                        } else {
                            format!("{}/{}", prefix, name)
                        };
                        if entry.mode().is_tree() {
                            let child_obj = r_repo
                                .find_object(entry.oid())
                                .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
                            let child = child_obj
                                .try_into_tree()
                                .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
                            walk(r_repo, child, &path, out)?;
                        } else {
                            out.insert(path, entry.oid().into());
                        }
                    }
                    Ok(())
                }

                walk(&gix_repo, tree, "", &mut entries)?;
                Ok(entries)
            };

        let entries_a = get_tree_entries(a)?;
        let entries_b = get_tree_entries(b)?;

        let mut diffs = Vec::new();

        for (path, oid_b) in &entries_b {
            if let Some(oid_a) = entries_a.get(path) {
                if oid_a != oid_b {
                    diffs.push(FileDiffStat {
                        path: path.clone(),
                        additions: 1,
                        deletions: 1,
                    });
                }
            } else {
                diffs.push(FileDiffStat {
                    path: path.clone(),
                    additions: 1,
                    deletions: 0,
                });
            }
        }

        for path in entries_a.keys() {
            if !entries_b.contains_key(path) {
                diffs.push(FileDiffStat {
                    path: path.clone(),
                    additions: 0,
                    deletions: 1,
                });
            }
        }

        Ok(diffs)
    }

    fn delete_local_branch(&self, _repo: &Repository, _branch: &BranchName) -> Result<()> {
        Err(crate::GitPurgeError::BackendUnsupported(
            "delete_local_branch not implemented in GixBackend".to_string(),
        ))
    }

    fn delete_remote_branch(
        &self,
        _repo: &Repository,
        _remote: &str,
        _branch: &BranchName,
    ) -> Result<()> {
        Err(crate::GitPurgeError::BackendUnsupported(
            "delete_remote_branch not implemented in GixBackend".to_string(),
        ))
    }

    fn create_ref(
        &self,
        _repo: &Repository,
        _full_ref: &str,
        _target: &Oid,
        _force: bool,
    ) -> Result<()> {
        Err(crate::GitPurgeError::BackendUnsupported(
            "create_ref not implemented in GixBackend".to_string(),
        ))
    }

    fn fetch(&self, _repo: &Repository, _remote: &str) -> Result<()> {
        Err(crate::GitPurgeError::BackendUnsupported(
            "fetch not implemented in GixBackend".to_string(),
        ))
    }
}

fn walk_tree_recursive(
    gix_repo: &gix::Repository,
    tree: gix::Tree<'_>,
    current_path: &str,
    paths: &mut Vec<String>,
) -> Result<()> {
    for entry in tree.iter() {
        let entry = entry.map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
        let name = entry.filename().to_string();
        let path = if current_path.is_empty() {
            name
        } else {
            format!("{}/{}", current_path, name)
        };

        if entry.mode().is_tree() {
            let child_obj = gix_repo
                .find_object(entry.oid())
                .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            let child_tree = child_obj
                .try_into_tree()
                .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
            walk_tree_recursive(gix_repo, child_tree, &path, paths)?;
        } else {
            paths.push(path);
        }
    }
    Ok(())
}

fn read_commit_internal(gix_repo: &gix::Repository, oid: gix::hash::ObjectId) -> Result<Commit> {
    let obj = gix_repo
        .find_object(oid)
        .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
    let commit = obj
        .try_into_commit()
        .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;
    let commit_decoded = commit
        .decode()
        .map_err(|e| crate::GitPurgeError::Git(e.to_string()))?;

    let author_sig = commit_decoded.author();
    let committer_sig = commit_decoded.committer();

    let author = Signature {
        name: author_sig.name.to_string(),
        email: author_sig.email.to_string(),
        when: convert_time(author_sig.time),
    };

    let committer = Signature {
        name: committer_sig.name.to_string(),
        email: committer_sig.email.to_string(),
        when: convert_time(committer_sig.time),
    };

    let author_date = author.when;
    let commit_date = committer.when;

    let subject = commit_decoded.message().summary().to_string();
    let parents = commit_decoded
        .parents()
        .map(|p| Oid(p.to_string()))
        .collect();

    Ok(Commit {
        oid: Oid(oid.to_string()),
        short: oid.to_hex_with_len(7).to_string(),
        author,
        committer,
        author_date,
        commit_date,
        subject,
        parents,
    })
}

fn convert_time(time: gix::date::Time) -> time::OffsetDateTime {
    let utc_time = time::OffsetDateTime::from_unix_timestamp(time.seconds)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
    let offset = time::UtcOffset::from_whole_seconds(time.offset).unwrap_or(time::UtcOffset::UTC);
    utc_time.to_offset(offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BranchName, RefSpec};
    use crate::testkit;

    #[test]
    fn test_gix_backend_read_operations() {
        let repo_fixture = testkit::merged_repo();
        let backend = GixBackend;

        let repo_model = Repository {
            id: crate::model::RepoId("test".to_string()),
            display_name: "test".to_string(),
            local_path: Some(repo_fixture.path().to_path_buf()),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };

        // Test open
        assert!(backend.open_repo(&repo_model).is_ok());

        // Test list_refs
        let refs = backend.list_refs(&repo_model).unwrap();
        assert!(refs.iter().any(|r| r.short == "main"));
        assert!(refs.iter().any(|r| r.short == "merged-branch"));
        assert!(refs.iter().any(|r| r.short == "unmerged-branch"));

        // Test list_branches
        let branches = backend.list_branches(&repo_model, None).unwrap();
        assert!(branches.len() >= 3);

        let main_branch = branches.iter().find(|b| b.name.0 == "main").unwrap();
        assert!(main_branch.is_head);

        // Test resolve_ref
        let commit = backend
            .resolve_ref(
                &repo_model,
                &RefSpec::Branch(BranchName("main".to_string())),
            )
            .unwrap();
        assert!(commit.subject.contains("merged-branch"));

        // Test is_ancestor
        let merged_commit = backend
            .resolve_ref(
                &repo_model,
                &RefSpec::Branch(BranchName("merged-branch".to_string())),
            )
            .unwrap();
        let unmerged_commit = backend
            .resolve_ref(
                &repo_model,
                &RefSpec::Branch(BranchName("unmerged-branch".to_string())),
            )
            .unwrap();

        assert!(backend
            .is_ancestor(&repo_model, &merged_commit.oid, &commit.oid)
            .unwrap());
        assert!(!backend
            .is_ancestor(&repo_model, &unmerged_commit.oid, &commit.oid)
            .unwrap());

        // Test merge_base
        let base = backend
            .merge_base(&repo_model, &merged_commit.oid, &unmerged_commit.oid)
            .unwrap();
        assert!(base.is_some());

        // Test read_tree
        let files = backend
            .read_tree(
                &repo_model,
                &RefSpec::Branch(BranchName("main".to_string())),
            )
            .unwrap();
        assert!(files.contains(&"file.txt".to_string()));
        assert!(files.contains(&"file_merged.txt".to_string()));

        // Test read_blob
        let content = backend
            .read_blob(
                &repo_model,
                &RefSpec::Branch(BranchName("main".to_string())),
                "file.txt",
            )
            .unwrap();
        assert_eq!(content, b"initial content");

        // Test diff_refs
        let diffs = backend
            .diff_refs(
                &repo_model,
                &RefSpec::Branch(BranchName("main".to_string())),
                &RefSpec::Branch(BranchName("unmerged-branch".to_string())),
            )
            .unwrap();
        // unmerged-branch has file_unmerged.txt added compared to main
        assert!(diffs
            .iter()
            .any(|d| d.path == "file_unmerged.txt" && d.additions == 1 && d.deletions == 0));
    }
}
