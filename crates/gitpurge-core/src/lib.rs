//! # gitpurge-core
//!
//! The shared **brain** of Git Purge: all domain logic for safely purging stale git
//! branches lives here. The CLI (`gitpurge-cli`) and the desktop backend
//! (`gitpurge-desktop`) are thin adapters that both call this crate — so "the CLI and
//! the UI share the same behavior" is guaranteed by the compiler, not by discipline
//! (see `docs/02-architecture.md` §1 and `delivery/CONVENTIONS.md` §2).
//!
//! ## Design
//!
//! `gitpurge-core` is a hexagonal (ports & adapters) library:
//!
//! - **Domain model** ([`model`]) — the vocabulary: repositories, branches, commits,
//!   classifications, policies, snapshots, plans, reports.
//! - **Ports (traits)** — [`git::GitBackend`], [`auth::SecretStore`],
//!   [`history::HistoryStore`], [`report::ReportSink`], [`clock::Clock`],
//!   [`progress::ProgressSink`] — every external concern is a trait so tests can
//!   substitute deterministic fakes.
//! - **Services / facade** — [`Engine`] orchestrates the use-cases (scan, plan,
//!   backup, execute, restore, diff, report, history) over the injected ports.
//!
//! ## Safety model (see `docs/11-safety-model.md`)
//!
//! Dry-run is the default for every mutating operation; destructive operations are
//! preceded by a verified backup snapshot; protected refs are structurally excluded
//! from destructive plans. These invariants are enforced in the service layer, never
//! left to callers.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod action;
pub mod auth;
pub mod backup;
pub mod clock;
pub mod config;
pub mod diff;
pub mod error;
pub mod git;
pub mod history;
pub mod model;
pub mod policy;
pub mod progress;
pub mod report;
pub mod scan;

#[cfg(feature = "testkit")]
pub mod testkit;

pub use config::Config;
pub use error::{GitPurgeError, Result};

use std::path::Path;

use crate::model::{
    ActionFilter, BackupOptions, ExecMode, Plan, RefSpec, RepoId, RestoreOutcome, RestoreSpec,
    RunReport, ScanOptions, ScanResult, Snapshot, SnapshotId,
};
use crate::report::ReportFormat;

/// The public facade over `gitpurge-core`.
///
/// Both the CLI and the Tauri backend construct an [`Engine`] and drive the entire
/// product through these methods (the "shared-core contract", `docs/02` §4). The
/// engine holds the resolved [`Config`] and the injected ports (git backend, secret
/// store, history store, report sink, clock, progress sink).
///
/// `Engine` is `Send + Sync` so it can be shared across async tasks and Tauri command
/// handlers. Long-running operations will additionally accept a `ProgressSink` and a
/// cancellation token once implemented (see `docs/02` §5).
#[derive(Debug)]
pub struct Engine {
    // TODO(P0-T4): replace with boxed port fields:
    //   git: Box<dyn git::GitBackend>,
    //   secrets: Box<dyn auth::SecretStore>,
    //   history: Box<dyn history::HistoryStore>,
    //   report_sink: Box<dyn report::ReportSink>,
    //   clock: Box<dyn clock::Clock>,
    //   progress: Box<dyn progress::ProgressSink>,
    // Ports are injected so tests can substitute in-memory fakes (docs/02 §3).
    #[allow(dead_code)]
    config: Config,
}

// Compile-time assertion: Engine must be Send + Sync because all port traits
// require Send + Sync. If this fails to compile, a port trait is missing the bound.
const _: () = {
    fn _assert_send_sync<T: Send + Sync>() {}
    fn _check() {
        _assert_send_sync::<Engine>();
    }
};

impl Engine {
    /// Open an engine with the given configuration, wiring up the default production
    /// adapters (gix/git2 git backend, keyring secret store, SQLite history store).
    ///
    /// # Errors
    /// Returns [`GitPurgeError`] if storage locations cannot be resolved/created or a
    /// port fails to initialize.
    pub fn open(config: Config) -> Result<Self> {
        // TODO(P0-T4): construct and inject the production ports here.
        Ok(Self { config })
    }

    /// Classify the branches of a repository (read-only). See [`scan`].
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from the git backend or policy engine.
    pub fn scan(&self, repo: &RepoId, opts: ScanOptions) -> Result<ScanResult> {
        let _ = (repo, opts);
        // TODO(P1-T4): delegate to scan::Scanner over the injected GitBackend + Policy.
        todo!("scan pipeline — phase P1")
    }

    /// Resolve the set of actions a delete/archive command *would* take (dry-run).
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from scanning or policy evaluation.
    pub fn plan(&self, repo: &RepoId, filter: &ActionFilter) -> Result<Plan> {
        let _ = (repo, filter);
        // TODO(P3-T1): build a Plan (never mutating), excluding protected refs.
        todo!("plan resolution — phase P3")
    }

    /// Create a point-in-time backup snapshot of a repository's refs.
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from the backup subsystem.
    pub fn backup_create(&self, repo: &RepoId, opts: BackupOptions) -> Result<Snapshot> {
        let _ = (repo, opts);
        // TODO(P2-T1): namespaced-ref snapshot into the shared bare mirror.
        todo!("backup snapshot — phase P2")
    }

    /// Execute a previously-resolved [`Plan`]. In [`ExecMode::DryRun`] this touches
    /// nothing; in [`ExecMode::Execute`] it backs up first, then acts, with
    /// auto-restore on failure.
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from backup or the git backend.
    pub fn execute(&self, plan: &Plan, mode: ExecMode) -> Result<RunReport> {
        let _ = (plan, mode);
        // TODO(P3-T2): safety-first execution wrapper (backup -> act -> record).
        todo!("plan execution — phase P3")
    }

    /// Restore a ref (as a branch or a tag) from a snapshot.
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`]; never silently overwrites an existing ref.
    pub fn restore(&self, snap: &SnapshotId, spec: RestoreSpec) -> Result<RestoreOutcome> {
        let _ = (snap, spec);
        // TODO(P2-T2): restore-as-branch / restore-as-tag with consent.
        todo!("restore — phase P2")
    }

    /// Diff two refs.
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from the git backend.
    pub fn diff(&self, a: &RefSpec, b: &RefSpec) -> Result<crate::diff::DiffResult> {
        let _ = (a, b);
        // TODO(P1-T5): compute a DiffResult via the GitBackend.
        todo!("diff — phase P1")
    }

    /// View the tree (or a single path) at a ref/commit.
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from the git backend.
    pub fn show_tree(&self, at: &RefSpec, path: Option<&Path>) -> Result<crate::diff::TreeView> {
        let _ = (at, path);
        // TODO(P1-T5): read the object DB tree at `at`.
        todo!("show_tree — phase P1")
    }

    /// Generate an audit/trend report in the requested format.
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from reporting or history.
    pub fn report(&self, repo: &RepoId, fmt: ReportFormat) -> Result<crate::report::Report> {
        let _ = (repo, fmt);
        // TODO(P5-T1): render md/json/html via a ReportSink.
        todo!("report — phase P5")
    }

    /// Fetch the recorded trend history for a repository.
    ///
    /// # Errors
    /// Propagates any [`GitPurgeError`] from the history store.
    pub fn history(&self, repo: &RepoId) -> Result<crate::history::TrendHistory> {
        let _ = repo;
        // TODO(P5-T2): query the SQLite HistoryStore.
        todo!("history — phase P5")
    }
}
