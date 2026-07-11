//! Action orchestration — delete, archive, restore (docs/04 §3).
//!
//! This module wires together the safety model: dry-run default, backup-before-destroy,
//! auto-restore on failure. It delegates actual git operations to `GitBackend`.

pub mod guard;
pub use guard::execute_deletions_with_guard;
