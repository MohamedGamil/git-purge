//! Error types for `gitpurge-core` (CONVENTIONS §11).
//!
//! Public functions return [`Result<T>`] = `Result<T, GitPurgeError>`. The variants
//! here are the *typed* domain errors; adapters project them outward:
//!
//! - the **CLI** maps each variant to an exit code and a `miette`/`anstyle` rendered
//!   message (see `gitpurge-cli`);
//! - the **Tauri backend** projects them into a serde-friendly `SerializableError`
//!   (`code` + `message` + `hint`) returned to the Vue frontend.
//!
//! No secrets or PII ever appear in an error message (CONVENTIONS §7, SAFE-07).

use thiserror::Error;

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, GitPurgeError>;

/// The typed error enum for all of `gitpurge-core`.
///
/// Representative variants only — more will be added as subsystems land. Each variant
/// carries enough context to render a helpful, secret-free message.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GitPurgeError {
    /// A tracked repository could not be found by id.
    #[error("repository not found: {0}")]
    RepoNotFound(String),

    /// A referenced branch/tag/ref does not exist or could not be read.
    #[error("reference not found: {0}")]
    RefNotFound(String),

    /// The operation was refused by the safety model (e.g. touching a protected ref,
    /// or a destructive op without confirmation / backup).
    #[error("blocked by safety policy: {0}")]
    SafetyViolation(String),

    /// A requested backup snapshot was missing or failed verification.
    #[error("snapshot error: {0}")]
    Snapshot(String),

    /// Configuration could not be loaded, parsed, or validated.
    #[error("configuration error: {0}")]
    Config(String),

    /// Authentication / credential retrieval failed. Never contains the secret.
    #[error("authentication error: {0}")]
    Auth(String),

    /// A git backend operation failed.
    #[error("git backend error: {0}")]
    Git(String),

    /// The underlying history/trends store failed.
    #[error("history store error: {0}")]
    History(String),

    /// A filesystem or IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// TODO(P1-T2): add `#[from]` conversions for gix / git2 / rusqlite / keyring /
    /// toml / serde_json error types as those adapters are implemented, so `?` works
    /// ergonomically in the service layer. Kept minimal here to avoid pulling those
    /// error types into the public surface prematurely.
    #[error("{0}")]
    Other(String),
}

impl GitPurgeError {
    /// Stable, machine-readable error code used by the CLI (exit-code mapping) and the
    /// Tauri `SerializableError` projection.
    ///
    // TODO(P3-T3): finalize the exit-code table in the CLI and keep it in sync here.
    pub fn code(&self) -> &'static str {
        match self {
            Self::RepoNotFound(_) => "repo_not_found",
            Self::RefNotFound(_) => "ref_not_found",
            Self::SafetyViolation(_) => "safety_violation",
            Self::Snapshot(_) => "snapshot_error",
            Self::Config(_) => "config_error",
            Self::Auth(_) => "auth_error",
            Self::Git(_) => "git_error",
            Self::History(_) => "history_error",
            Self::Io(_) => "io_error",
            Self::Other(_) => "error",
        }
    }
}
