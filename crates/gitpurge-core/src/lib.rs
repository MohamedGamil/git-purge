//! # gitpurge-core
//!
//! The shared **brain** of Git Purge: all domain logic for safely purging stale git
//! branches lives here. The CLI (`gitpurge-cli`) and the desktop backend
//! (`gitpurge-desktop`) are thin adapters that both call this crate — so "the CLI and
//! the UI share the same behavior" is guaranteed by the compiler, not by discipline
//! (see `docs/02-architecture.md` §1 and `delivery/CONVENTIONS.md` §2).

#![deny(unsafe_code)]
#![warn(missing_docs)]

/// Domain action model.
pub mod action;
/// Domain authentication and credential storage ports.
pub mod auth;
/// Domain backup snapshot model.
pub mod backup;
/// Clock port.
pub mod clock;
/// Configuration loading and persistence.
pub mod config;
/// Diff and tree-view domain types.
pub mod diff;
/// Error definitions and conversion logic.
pub mod error;
/// Git operations port and adapter implementations.
pub mod git;
/// History and trend tracking.
pub mod history;
/// Core domain entities, value objects, and states.
pub mod model;
/// Policy compiler and rule evaluators.
pub mod policy;
/// Progress monitoring.
pub mod progress;
/// Reporting services.
pub mod report;
/// Classification pipeline.
pub mod scan;

/// Engine implementations (P10-T2).
pub mod engine;

#[cfg(any(test, feature = "testkit"))]
/// Fixture builders and test harness helpers.
pub mod testkit;

pub use config::Config;
pub use engine::Engine;
pub use error::{GitPurgeError, Result};
pub use git::GitBackend;

/// Log an operation (delete or archive) to a file for debugging.
pub fn log_operation(op: &str, branch: &str, scope: &str, result: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let log_dir = if let Some(bd) = directories::BaseDirs::new() {
        bd.home_dir().join(".gitpurge")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".gitpurge")
    } else {
        std::env::temp_dir().join(".gitpurge")
    };

    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("git-purge-operations.log");

    let now = time::OffsetDateTime::now_utc().to_string();
    let log_line = format!(
        "[{}] OP: {} | BRANCH: {} | SCOPE: {} | RESULT: {}\n",
        now, op, branch, scope, result
    );
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = file.write_all(log_line.as_bytes());
    }
}
