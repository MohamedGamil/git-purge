//! Backup and snapshot management (docs/08 backup-and-restore, CONVENTIONS §6).
//!
//! Snapshots are point-in-time captures of a repo's refs, stored as namespaced refs
//! inside a shared bare mirror (`refs/gitpurge/backups/<snapshot-id>/<original-ref>`).
//! N snapshots share the object database, so cost is ~O(changed objects), not O(N × repo).

// TODO(P2-T1): implement BackupManager (create, list, show, verify, prune).
// TODO(P2-T2): implement restore-as-branch and restore-as-tag with consent.
