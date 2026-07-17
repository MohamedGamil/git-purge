//! Backup & snapshot methods (P10-T2).

use crate::error::Result;
use crate::model::{BackupOptions, ExecMode, RepoId, ScanOptions, Snapshot, SnapshotId};

impl super::Engine {
    /// Create a point-in-time backup snapshot of a repository's refs.
    pub fn backup_create(&self, repo_id: &RepoId, opts: BackupOptions) -> Result<Snapshot> {
        let repo = {
            let repos = self.repos.lock().unwrap();
            repos.get(repo_id).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!(
                    "Repository not registered: {:?}",
                    repo_id
                ))
            })?
        };

        let scan_opts = ScanOptions::default();
        let scan_res = self.scan(repo_id, scan_opts)?;

        let config_guard = self.config.lock().unwrap();
        let snapshot = crate::backup::create_snapshot(
            &config_guard,
            self.git.as_ref(),
            &repo,
            &scan_res.classifications,
            &opts,
        )?;

        let verify_report =
            crate::backup::verify_snapshot(&config_guard, repo_id, &snapshot.id, false)?;
        if !verify_report.ok {
            return Err(crate::GitPurgeError::Snapshot(format!(
                "Backup snapshot '{}' failed verification.",
                snapshot.id.0
            )));
        }

        let mut verified_snapshot = snapshot;
        verified_snapshot.verified = true;

        self.history.save_snapshot(&verified_snapshot)?;

        Ok(verified_snapshot)
    }

    /// List all backup snapshots for a repository.
    pub fn list_snapshots(&self, repo_id: &RepoId) -> Result<Vec<Snapshot>> {
        self.history.list_snapshots(repo_id)
    }

    /// Get details of a specific snapshot.
    pub fn get_snapshot(&self, snap_id: &SnapshotId) -> Result<Option<Snapshot>> {
        self.history.get_snapshot(snap_id)
    }

    /// Delete a snapshot's metadata.
    pub fn delete_snapshot(&self, snap_id: &SnapshotId) -> Result<()> {
        self.history.delete_snapshot(snap_id)
    }

    /// Verify the integrity of a backup snapshot in the bare mirror.
    pub fn backup_verify(
        &self,
        repo_id: &RepoId,
        snap_id: &SnapshotId,
    ) -> Result<crate::backup::VerifyReport> {
        crate::backup::verify_snapshot(&self.config.lock().unwrap(), repo_id, snap_id, false)
    }

    /// Prune old snapshots for a repository based on a retention policy.
    pub fn backup_prune(
        &self,
        repo_id: &RepoId,
        policy: &crate::model::RetentionPolicy,
        mode: ExecMode,
    ) -> Result<crate::model::PruneReport> {
        crate::backup::prune_snapshots(
            &self.config.lock().unwrap(),
            self.history.as_ref(),
            repo_id,
            policy,
            mode,
        )
    }

    /// Purge backups bare mirror for a repository from disk.
    pub fn purge_repo_backups(&self, id: &RepoId) -> Result<()> {
        let config_guard = self.config.lock().unwrap();
        let mirror_manager = crate::backup::BackupMirrorManager::new(&config_guard);
        let mirror_path = mirror_manager.resolve_mirror_path(id);
        if mirror_path.exists() {
            std::fs::remove_dir_all(mirror_path).map_err(|e| {
                crate::GitPurgeError::Git(format!("Failed to delete bare mirror directory: {}", e))
            })?;
        }
        Ok(())
    }
}
