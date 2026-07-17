//! Credential storage port (docs/02 §3, docs/09 authentication).
//!
//! `SecretStore` abstracts credential storage so `gitpurge-core` never depends on
//! a specific keychain or encryption implementation. Production uses `keyring` with
//! an encrypted-file fallback; tests use `FakeSecretStore`.
//!
//! **SAFE-07**: no secret material ever appears in logs, errors, snapshots, or reports.

mod credential;
pub mod file_store;
pub mod keyring_store;

pub use credential::{Credential, CredentialEntry, CredentialKind, CredentialQuery};
pub use file_store::FileSecretStore;
pub use keyring_store::KeyringSecretStore;

use crate::error::Result;
use crate::model::RepoId;
use std::collections::HashMap;
use std::sync::Mutex;

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
    fn list(&self) -> Result<Vec<CredentialEntry>>;

    /// Test that a stored credential can authenticate to the remote.
    fn test(&self, repo: &RepoId, remote: &str) -> Result<bool>;
}

type StoreKey = (String, String);

#[derive(Clone)]
struct StoredCredential {
    kind: CredentialKind,
    label: String,
    secret: Vec<u8>,
}

impl std::fmt::Debug for StoredCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredCredential")
            .field("kind", &self.kind)
            .field("label", &self.label)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

/// In-memory fake for tests. Never logs or exposes secrets outside [`Credential::secret`].
#[derive(Debug, Default)]
pub struct FakeSecretStore {
    entries: Mutex<HashMap<StoreKey, StoredCredential>>,
}

impl FakeSecretStore {
    fn key(repo: &RepoId, remote: &str) -> StoreKey {
        (repo.0.clone(), remote.to_string())
    }

    fn label_for(kind: CredentialKind, remote: &str) -> String {
        format!("{kind:?} for {remote}")
    }
}

impl SecretStore for FakeSecretStore {
    fn store(
        &self,
        repo: &RepoId,
        remote: &str,
        kind: CredentialKind,
        secret: &[u8],
    ) -> Result<()> {
        let label = Self::label_for(kind, remote);
        self.entries.lock().unwrap().insert(
            Self::key(repo, remote),
            StoredCredential {
                kind,
                label,
                secret: secret.to_vec(),
            },
        );
        Ok(())
    }

    fn retrieve(&self, repo: &RepoId, remote: &str) -> Result<Option<Credential>> {
        let entry = self
            .entries
            .lock()
            .unwrap()
            .get(&Self::key(repo, remote))
            .cloned();
        Ok(entry.map(|stored| Credential::new(stored.kind, stored.label, stored.secret)))
    }

    fn remove(&self, repo: &RepoId, remote: &str) -> Result<()> {
        self.entries
            .lock()
            .unwrap()
            .remove(&Self::key(repo, remote));
        Ok(())
    }

    fn list(&self) -> Result<Vec<CredentialEntry>> {
        let entries = self.entries.lock().unwrap();
        Ok(entries
            .iter()
            .map(|((repo, remote), stored)| CredentialEntry {
                repo: RepoId(repo.clone()),
                remote: remote.clone(),
                kind: stored.kind,
                label: stored.label.clone(),
            })
            .collect())
    }

    fn test(&self, _repo: &RepoId, _remote: &str) -> Result<bool> {
        Ok(true)
    }
}
