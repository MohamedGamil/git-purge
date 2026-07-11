//! Backup and snapshot management (docs/08 backup-and-restore, CONVENTIONS §6).
//!
//! Snapshots are point-in-time captures of a repo's refs, stored as namespaced refs
//! inside a shared bare mirror (`refs/gitpurge/backups/<snapshot-id>/<original-ref>`).
//! N snapshots share the object database, so cost is ~O(changed objects), not O(N × repo).

pub mod mirror;
pub mod snapshot;
pub mod verify;
pub mod prune;
pub mod restore;

pub use mirror::BackupMirrorManager;
pub use snapshot::create_snapshot;
pub use verify::{verify_snapshot, VerifyReport, VerifyProblem, RefCheck};
pub use prune::prune_snapshots;
pub use restore::restore_snapshot;
