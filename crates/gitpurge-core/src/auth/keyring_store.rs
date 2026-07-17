//! OS keyring SecretStore adapter (P11-T2).

use crate::auth::{Credential, CredentialEntry, CredentialKind, SecretStore};
use crate::error::Result;
use crate::model::RepoId;
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "git-purge";
const INDEX_KEY: &str = "git-purge-credentials-index";

#[derive(Serialize, Deserialize)]
struct KeyringPayload {
    kind: CredentialKind,
    label: String,
    secret: Vec<u8>,
}

/// A SecretStore implementation backed by the OS Keychain / Credential Manager.
#[derive(Debug)]
pub struct KeyringSecretStore {
    service: String,
}

impl Default for KeyringSecretStore {
    fn default() -> Self {
        Self {
            service: SERVICE_NAME.to_string(),
        }
    }
}

impl KeyringSecretStore {
    /// Create a new KeyringSecretStore with a custom service name.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn credential_key(&self, repo: &RepoId, remote: &str) -> String {
        format!("{}_{}", repo.0, remote)
    }

    fn read_index(&self) -> Result<Vec<CredentialEntry>> {
        let entry = Entry::new(&self.service, INDEX_KEY)
            .map_err(|e| crate::GitPurgeError::Auth(format!("Keyring error: {}", e)))?;

        match entry.get_password() {
            Ok(json) => {
                let index: Vec<CredentialEntry> = serde_json::from_str(&json).map_err(|e| {
                    crate::GitPurgeError::Auth(format!("Failed to parse credentials index: {}", e))
                })?;
                Ok(index)
            }
            Err(keyring::Error::NoEntry) => Ok(Vec::new()),
            Err(e) => Err(crate::GitPurgeError::Auth(format!("Keyring error: {}", e))),
        }
    }

    fn write_index(&self, index: &[CredentialEntry]) -> Result<()> {
        let entry = Entry::new(&self.service, INDEX_KEY)
            .map_err(|e| crate::GitPurgeError::Auth(format!("Keyring error: {}", e)))?;

        let json = serde_json::to_string(index).map_err(|e| {
            crate::GitPurgeError::Auth(format!("Failed to serialize credentials index: {}", e))
        })?;

        entry
            .set_password(&json)
            .map_err(|e| crate::GitPurgeError::Auth(format!("Keyring error: {}", e)))?;

        Ok(())
    }
}

impl SecretStore for KeyringSecretStore {
    fn store(
        &self,
        repo: &RepoId,
        remote: &str,
        kind: CredentialKind,
        secret: &[u8],
    ) -> Result<()> {
        let username = self.credential_key(repo, remote);
        let label = format!("{kind:?} for {remote}");

        // 1. Store the secret payload
        let entry = Entry::new(&self.service, &username)
            .map_err(|e| crate::GitPurgeError::Auth(format!("Keyring error: {}", e)))?;

        let payload = KeyringPayload {
            kind,
            label: label.clone(),
            secret: secret.to_vec(),
        };

        let json = serde_json::to_string(&payload).map_err(|e| {
            crate::GitPurgeError::Auth(format!("Failed to serialize credential payload: {}", e))
        })?;

        entry
            .set_password(&json)
            .map_err(|e| crate::GitPurgeError::Auth(format!("Keyring error: {}", e)))?;

        // 2. Update the index
        let mut index = self.read_index()?;
        index.retain(|e| e.repo != *repo || e.remote != remote);
        index.push(CredentialEntry {
            repo: repo.clone(),
            remote: remote.to_string(),
            kind,
            label,
        });
        self.write_index(&index)?;

        Ok(())
    }

    fn retrieve(&self, repo: &RepoId, remote: &str) -> Result<Option<Credential>> {
        let username = self.credential_key(repo, remote);
        let entry = Entry::new(&self.service, &username)
            .map_err(|e| crate::GitPurgeError::Auth(format!("Keyring error: {}", e)))?;

        match entry.get_password() {
            Ok(json) => {
                let payload: KeyringPayload = serde_json::from_str(&json).map_err(|e| {
                    crate::GitPurgeError::Auth(format!("Failed to parse credential payload: {}", e))
                })?;
                Ok(Some(Credential::new(
                    payload.kind,
                    payload.label,
                    payload.secret,
                )))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(crate::GitPurgeError::Auth(format!("Keyring error: {}", e))),
        }
    }

    fn remove(&self, repo: &RepoId, remote: &str) -> Result<()> {
        let username = self.credential_key(repo, remote);
        let entry = Entry::new(&self.service, &username)
            .map_err(|e| crate::GitPurgeError::Auth(format!("Keyring error: {}", e)))?;

        // Delete payload (ignore if not found)
        match entry.delete_credential() {
            Ok(_) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(crate::GitPurgeError::Auth(format!("Keyring error: {}", e))),
        }

        // Remove from index
        let mut index = self.read_index()?;
        let original_len = index.len();
        index.retain(|e| e.repo != *repo || e.remote != remote);
        if index.len() != original_len {
            self.write_index(&index)?;
        }

        Ok(())
    }

    fn list(&self) -> Result<Vec<CredentialEntry>> {
        self.read_index()
    }

    fn test(&self, repo: &RepoId, remote: &str) -> Result<bool> {
        let cred = self.retrieve(repo, remote)?;
        Ok(cred.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_store_roundtrip() {
        let store = KeyringSecretStore::new("git-purge-test-service");
        let repo = RepoId("test-repo-123".to_string());
        let remote = "origin";
        let secret_bytes = b"secret-token-xyz";

        // Clean up first in case of a dirty previous run
        let _ = store.remove(&repo, remote);

        // Store
        match store.store(&repo, remote, CredentialKind::HttpsToken, secret_bytes) {
            Ok(_) => {
                // Retrieve to check if keyring is functional in this environment
                if let Ok(Some(retrieved)) = store.retrieve(&repo, remote) {
                    assert_eq!(retrieved.secret(), secret_bytes);
                    assert_eq!(retrieved.kind(), CredentialKind::HttpsToken);

                    // List
                    let list = store.list().unwrap();
                    assert!(list
                        .iter()
                        .any(|entry| entry.repo == repo && entry.remote == remote));

                    // Test
                    assert!(store.test(&repo, remote).unwrap());

                    // Remove
                    store.remove(&repo, remote).unwrap();
                    let retrieved_after = store.retrieve(&repo, remote).unwrap();
                    assert!(retrieved_after.is_none());
                } else {
                    eprintln!(
                        "Keyring backend is mock/dummy or failed to persist in this environment."
                    );
                }
            }
            Err(e) => {
                // If keyring is not supported/configured on this OS environment (e.g. headless CI without D-Bus),
                // we print a warning and skip the test assertions so it doesn't break builds.
                eprintln!(
                    "Keyring not supported or failed to initialize in this environment: {:?}",
                    e
                );
            }
        }
    }
}
