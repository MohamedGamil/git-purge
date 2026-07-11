//! Snapshot creation (P2-T2, CONVENTIONS §6, docs/08 §4).

use time::OffsetDateTime;
use ulid::Ulid;
use crate::error::Result;
use crate::model::{Repository, Snapshot, SnapshotId, SnapshotRef, SnapshotTrigger, MergeState};
use crate::backup::mirror::BackupMirrorManager;

/// Create a new snapshot of the repository's refs and objects inside the bare mirror.
pub fn create_snapshot(
    config: &crate::config::Config,
    git_backend: &dyn crate::git::GitBackend,
    repo: &Repository,
    classifications: &[crate::model::Classification],
    opts: &crate::model::BackupOptions,
) -> Result<Snapshot> {
    let mirror_manager = BackupMirrorManager::new(config);
    let mirror_repo = mirror_manager.ensure_mirror(repo)?;

    let mut _stale_mirror = false;
    // 1. Fetch objects from source to bare mirror
    if let Err(e) = mirror_manager.fetch_to_mirror(repo, &mirror_repo) {
        tracing::warn!("Failed to fetch to mirror (offline?): {}", e);
        _stale_mirror = true;
    }

    // 2. Generate snapshot ID (ULID)
    let snapshot_id = SnapshotId(Ulid::new().to_string());
    let created_at = OffsetDateTime::now_utc();
    let trigger = opts.trigger.unwrap_or(SnapshotTrigger::Manual);

    // 3. Resolve source branches to back up
    let branches = git_backend.list_branches(repo, None)?;
    let mut snapshot_refs = Vec::new();

    // Start a transaction in the mirror repo to write refs atomically
    let mut tx = mirror_repo.transaction()
        .map_err(|e| crate::GitPurgeError::Git(format!("Failed to start ref transaction in mirror: {}", e)))?;

    for branch in branches {
        if !opts.only_branches.is_empty() && !opts.only_branches.contains(&branch.name) {
            continue;
        }

        let class = classifications.iter().find(|c| c.branch == branch.name && c.scope == branch.scope);
        let merged_at_capture = class.map(|c| c.merge_state).unwrap_or(MergeState::Unknown);

        let backup_ref_path = format!(
            "refs/gitpurge/backups/{}/{}",
            snapshot_id.0,
            branch.full_ref
        );

        let tip_oid = git2::Oid::from_str(&branch.tip.oid.0)
            .map_err(|e| crate::GitPurgeError::Git(format!("Invalid OID: {}", e)))?;

        tx.lock_ref(&backup_ref_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to lock backup ref {}: {}", backup_ref_path, e)))?;

        tx.set_target(&backup_ref_path, tip_oid, None, "Git Purge snapshot create")
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to set target for backup ref {}: {}", backup_ref_path, e)))?;

        // Walk and count reachable commits
        let mut revwalk = mirror_repo.revwalk()
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to get revwalk: {}", e)))?;
        revwalk.push(tip_oid)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to push tip OID to revwalk: {}", e)))?;
        let commit_count = revwalk.count() as u64;

        let upstream = branch.upstream.map(|u| format!("{}/{}", u.remote, u.ref_name.0));

        snapshot_refs.push(SnapshotRef {
            branch: branch.name,
            original_full_ref: branch.full_ref,
            backup_ref: backup_ref_path,
            tip: branch.tip.oid,
            commit_count,
            upstream,
            merged_at_capture,
        });
    }

    // Commit the transaction of refs
    tx.commit()
        .map_err(|e| crate::GitPurgeError::Git(format!("Failed to commit ref transaction in mirror: {}", e)))?;

    // 4. Construct the metadata manifest JSON
    let manifest_path = mirror_manager.resolve_mirror_path(&repo.id)
        .join(format!("{}.json", snapshot_id.0));

    let snapshot = Snapshot {
        id: snapshot_id.clone(),
        repo: repo.id.clone(),
        created_at,
        trigger,
        refs: snapshot_refs,
        verified: false,
        manifest_path: manifest_path.clone(),
    };

    // Serialize manifest to JSON bytes
    let json_bytes = serde_json::to_vec_pretty(&snapshot)
        .map_err(|e| crate::GitPurgeError::Config(format!("Failed to serialize snapshot manifest: {}", e)))?;

    // Save manifest as a git blob inside the mirror repo
    let blob_oid = mirror_repo.blob(&json_bytes)
        .map_err(|e| crate::GitPurgeError::Git(format!("Failed to write manifest blob to mirror: {}", e)))?;

    // Create the meta reference pointing to the blob
    let meta_ref_path = format!("refs/gitpurge/meta/{}", snapshot_id.0);
    mirror_repo.reference(&meta_ref_path, blob_oid, true, "Git Purge snapshot metadata")
        .map_err(|e| crate::GitPurgeError::Git(format!("Failed to create metadata reference: {}", e)))?;

    // Also write it to the filesystem inside the bare repository directory for redundancy/ease of manual audit
    std::fs::write(&manifest_path, &json_bytes)?;

    Ok(snapshot)
}
