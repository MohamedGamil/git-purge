//! Action orchestration — delete, archive, restore (docs/04 §3).
//!
//! This module wires together the safety model: dry-run default, backup-before-destroy,
//! auto-restore on failure. It delegates actual git operations to `GitBackend`.

// TODO(P3-T1): implement plan resolution (dry-run).
// TODO(P3-T2): implement safety-first execution wrapper (backup → act → record).
