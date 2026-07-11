//! Credential storage port (docs/02 §3, docs/09 authentication).
//!
//! `SecretStore` abstracts credential storage so `gitpurge-core` never depends on
//! a specific keychain or encryption implementation. Production uses `keyring` with
//! an encrypted-file fallback; tests use `FakeSecretStore`.
//!
//! **SAFE-07**: no secret material ever appears in logs, errors, snapshots, or reports.

use crate::error::Result;
use crate::model::RepoId;

/// A credential retrieved from storage.
#[derive(Debug, Clone)]
pub struct Credential {
    /// The kind of credential.
    pub kind: CredentialKind,
    /// An opaque label for display (e.g. "GitHub token for origin") — never the secret.
    pub label: String,
}

/// The type of credential stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CredentialKind {
    /// SSH private key (path or in-memory).
    SshKey,
    /// HTTPS username + password.
    HttpsBasic,
    /// HTTPS bearer / personal-access token.
    HttpsToken,
    /// System SSH agent identity.
    SshAgent,
}

/// Port for secure credential storage and retrieval.
///
/// Implementations must be `Send + Sync` for shared `Engine` access.
///
/// # Safety invariant (SAFE-07)
/// No method may include secret material in error messages, `Debug` output, or logs.
pub trait SecretStore: Send + Sync + std::fmt::Debug {
    /// Store a credential for a repo + remote.
    fn store(&self, repo: &RepoId, remote: &str, kind: CredentialKind, secret: &[u8])
        -> Result<()>;

    /// Retrieve a credential for a repo + remote.
    fn retrieve(&self, repo: &RepoId, remote: &str) -> Result<Option<Credential>>;

    /// Remove a stored credential.
    fn remove(&self, repo: &RepoId, remote: &str) -> Result<()>;

    /// List all stored credentials (labels only, never secrets).
    fn list(&self) -> Result<Vec<Credential>>;

    /// Test that a stored credential can authenticate to the remote.
    fn test(&self, repo: &RepoId, remote: &str) -> Result<bool>;
}

/// In-memory fake for tests. Never stores real secrets.
#[derive(Debug, Default)]
pub struct FakeSecretStore {
    // TODO(P6): add fields for canned credentials.
}

impl SecretStore for FakeSecretStore {
    fn store(
        &self,
        _repo: &RepoId,
        _remote: &str,
        _kind: CredentialKind,
        _secret: &[u8],
    ) -> Result<()> {
        Ok(())
    }

    fn retrieve(&self, _repo: &RepoId, _remote: &str) -> Result<Option<Credential>> {
        Ok(None)
    }

    fn remove(&self, _repo: &RepoId, _remote: &str) -> Result<()> {
        Ok(())
    }

    fn list(&self) -> Result<Vec<Credential>> {
        Ok(Vec::new())
    }

    fn test(&self, _repo: &RepoId, _remote: &str) -> Result<bool> {
        Ok(true)
    }
}
