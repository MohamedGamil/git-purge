//! Snapshot verification (P2-T3, CONVENTIONS §7.5, docs/08 §5.2/§8.3).

use crate::backup::mirror::BackupMirrorManager;
use crate::error::Result;
use crate::model::{BranchName, Snapshot, SnapshotId};

/// Result of a single ref check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RefCheck {
    /// The short branch name.
    pub branch: BranchName,
    /// Whether verification passed for this ref.
    pub ok: bool,
    /// The specific problem found, if any.
    pub problem: Option<VerifyProblem>,
}

/// Verification problems that can occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VerifyProblem {
    /// The backup reference is missing.
    MissingRef,
    /// The reference tip SHA does not match the manifest.
    ShaMismatch,
    /// The target commit object is missing from the database.
    MissingObject,
    /// Connectivity check failed during deep walk.
    FsckError,
}

/// Integrity check report.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VerifyReport {
    /// Overall status (true if all checks passed).
    pub ok: bool,
    /// Status per ref.
    pub per_ref: Vec<RefCheck>,
    /// All problems detected.
    pub problems: Vec<VerifyProblem>,
}

/// Verify the integrity of a snapshot in the bare mirror.
pub fn verify_snapshot(
    config: &crate::config::Config,
    repo_id: &crate::model::RepoId,
    id: &SnapshotId,
    deep: bool,
) -> Result<VerifyReport> {
    let mirror_manager = BackupMirrorManager::new(config);
    let mirror_path = mirror_manager.resolve_mirror_path(repo_id);

    if !mirror_path.exists() {
        return Err(crate::GitPurgeError::Snapshot(format!(
            "Bare mirror directory not found: {:?}",
            mirror_path
        )));
    }

    let mirror_repo = git2::Repository::open_bare(&mirror_path)
        .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open bare mirror: {}", e)))?;

    // 1. Resolve meta ref and read manifest
    let meta_ref_path = format!("refs/gitpurge/meta/{}", id.0);
    let meta_ref = match mirror_repo.find_reference(&meta_ref_path) {
        Ok(r) => r,
        Err(_) => {
            return Ok(VerifyReport {
                ok: false,
                per_ref: Vec::new(),
                problems: vec![VerifyProblem::MissingRef],
            });
        }
    };

    let target_oid = meta_ref.target().ok_or_else(|| {
        crate::GitPurgeError::Snapshot("Meta reference points to no OID".to_string())
    })?;

    let blob = mirror_repo.find_blob(target_oid).map_err(|e| {
        crate::GitPurgeError::Snapshot(format!("Failed to find metadata blob: {}", e))
    })?;

    let snapshot: Snapshot = serde_json::from_slice(blob.content()).map_err(|e| {
        crate::GitPurgeError::Config(format!("Failed to parse metadata manifest: {}", e))
    })?;

    // 2. Check each ref
    let mut per_ref = Vec::new();
    let mut problems = Vec::new();

    for ref_entry in &snapshot.refs {
        let mut ok = true;
        let mut problem = None;

        // Check if ref exists
        match mirror_repo.find_reference(&ref_entry.backup_ref) {
            Ok(r) => {
                let actual_oid = r.target().map(|o| o.to_string()).unwrap_or_default();
                if actual_oid != ref_entry.tip.0 {
                    ok = false;
                    problem = Some(VerifyProblem::ShaMismatch);
                } else {
                    // Check if commit object is present
                    let target_git2_oid = r.target().unwrap();
                    match mirror_repo.find_commit(target_git2_oid) {
                        Ok(commit) => {
                            if deep {
                                // Deep walk connectivity check
                                let mut revwalk = mirror_repo.revwalk().map_err(|e| {
                                    crate::GitPurgeError::Git(format!(
                                        "Failed to get revwalk: {}",
                                        e
                                    ))
                                })?;
                                if revwalk.push(target_git2_oid).is_err()
                                    || revwalk.any(|c| c.is_err())
                                {
                                    ok = false;
                                    problem = Some(VerifyProblem::FsckError);
                                }
                            } else {
                                // Walk just the tree of this commit to verify basic reachability
                                if commit.tree().is_err() {
                                    ok = false;
                                    problem = Some(VerifyProblem::MissingObject);
                                }
                            }
                        }
                        Err(_) => {
                            ok = false;
                            problem = Some(VerifyProblem::MissingObject);
                        }
                    }
                }
            }
            Err(_) => {
                ok = false;
                problem = Some(VerifyProblem::MissingRef);
            }
        }

        if let Some(p) = problem {
            if !problems.contains(&p) {
                problems.push(p);
            }
        }

        per_ref.push(RefCheck {
            branch: ref_entry.branch.clone(),
            ok,
            problem,
        });
    }

    let overall_ok = per_ref.iter().all(|r| r.ok);

    Ok(VerifyReport {
        ok: overall_ok,
        per_ref,
        problems,
    })
}
