//! Snapshot restoration (P2-T5, CONVENTIONS §6, docs/08 §7).

use crate::backup::mirror::BackupMirrorManager;
use crate::error::Result;
use crate::model::{Repository, RestoreOutcome, RestoreSpec, Snapshot, SnapshotId};

/// Restore a reference from a snapshot back into the source repository.
pub fn restore_snapshot(
    config: &crate::config::Config,
    repo: &Repository,
    id: &SnapshotId,
    spec: &RestoreSpec,
) -> Result<RestoreOutcome> {
    let mirror_manager = BackupMirrorManager::new(config);
    let mirror_path = mirror_manager.resolve_mirror_path(&repo.id);

    if !mirror_path.exists() {
        return Err(crate::GitPurgeError::Snapshot(format!(
            "Bare mirror directory not found: {:?}",
            mirror_path
        )));
    }

    let mirror_repo = git2::Repository::open_bare(&mirror_path)
        .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open bare mirror: {}", e)))?;

    // 1. Load manifest JSON to find ref details
    let meta_ref_path = format!("refs/gitpurge/meta/{}", id.0);
    let meta_ref = mirror_repo.find_reference(&meta_ref_path).map_err(|e| {
        crate::GitPurgeError::Snapshot(format!("Snapshot meta reference not found: {}", e))
    })?;

    let target_oid = meta_ref.target().ok_or_else(|| {
        crate::GitPurgeError::Snapshot("Meta reference points to no OID".to_string())
    })?;

    let blob = mirror_repo.find_blob(target_oid).map_err(|e| {
        crate::GitPurgeError::Snapshot(format!("Failed to find metadata blob: {}", e))
    })?;

    let snapshot: Snapshot = serde_json::from_slice(blob.content()).map_err(|e| {
        crate::GitPurgeError::Config(format!("Failed to parse metadata manifest: {}", e))
    })?;

    // Find the ref matching the branch name in the spec
    let ref_entry = snapshot
        .refs
        .iter()
        .find(|r| {
            if let Some(ref orig_ref) = spec.original_ref {
                r.original_full_ref == *orig_ref
            } else {
                r.branch == spec.branch
            }
        })
        .ok_or_else(|| {
            crate::GitPurgeError::RefNotFound(format!(
                "Branch '{}' not found in snapshot '{}'",
                spec.branch.0, id.0
            ))
        })?;

    // 2. Resolve source repository and target ref path
    let source_path = repo.local_path.as_ref().ok_or_else(|| {
        crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
    })?;

    let source_repo = git2::Repository::open(source_path)
        .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open source repo: {}", e)))?;

    let target_name = spec
        .target_name
        .clone()
        .unwrap_or_else(|| ref_entry.branch.0.clone());
    let target_ref_path = if spec.as_tag {
        format!("refs/tags/{}", target_name)
    } else {
        format!("refs/heads/{}", target_name)
    };

    // 3. SAFE-06: Check if reference already exists and force is false
    if source_repo.find_reference(&target_ref_path).is_ok() && !spec.force {
        return Err(crate::GitPurgeError::SafetyViolation(format!(
            "Reference '{}' already exists in the repository. Overwriting without explicit consent is blocked (SAFE-06).",
            target_ref_path
        )));
    }

    // 4. Fetch the object from mirror into source repository
    let mut remote = source_repo
        .remote_anonymous(&mirror_path.to_string_lossy())
        .map_err(|e| {
            crate::GitPurgeError::Git(format!(
                "Failed to create anonymous remote for source repo: {}",
                e
            ))
        })?;

    let force_prefix = if spec.force { "+" } else { "" };
    let refspec = format!(
        "{}{}:{}",
        force_prefix, ref_entry.backup_ref, target_ref_path
    );

    let mut fetch_opts = git2::FetchOptions::new();
    remote
        .fetch(&[&refspec], Some(&mut fetch_opts), None)
        .map_err(|e| {
            crate::GitPurgeError::Git(format!("Failed to fetch restored ref from mirror: {}", e))
        })?;

    Ok(RestoreOutcome {
        snapshot: id.clone(),
        branch: spec.branch.clone(),
        created_ref: target_ref_path,
        as_tag: spec.as_tag,
        tip: ref_entry.tip.clone(),
    })
}
