//! Encrypted file SecretStore adapter (P11-T3).

use crate::auth::{Credential, CredentialEntry, CredentialKind, SecretStore};
use crate::error::{GitPurgeError, Result};
use crate::model::RepoId;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use zeroize::Zeroizing;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

#[derive(Serialize, Deserialize, Clone)]
struct FileCredentialEntry {
    repo: RepoId,
    remote: String,
    kind: CredentialKind,
    label: String,
    secret: Vec<u8>,
}

/// A SecretStore implementation backed by an AES-256-GCM encrypted JSON file.
#[derive(Debug)]
pub struct FileSecretStore {
    path: PathBuf,
    passphrase: Zeroizing<String>,
    // Mutex to serialize writes and avoid race conditions within the same process
    lock: Mutex<()>,
}

impl FileSecretStore {
    /// Create a new FileSecretStore pointing to the given path with the passphrase.
    pub fn new(path: impl Into<PathBuf>, passphrase: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            passphrase: Zeroizing::new(passphrase.into()),
            lock: Mutex::new(()),
        }
    }

    /// Helper to derive the AES key from password and salt.
    fn derive_key(&self, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
        let mut key = [0u8; 32];
        let argon2 = Argon2::default();
        argon2
            .hash_password_into(self.passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| GitPurgeError::Auth(format!("Key derivation failed: {}", e)))?;
        Ok(Zeroizing::new(key))
    }

    /// Read and decrypt the file.
    fn read_entries(&self) -> Result<Vec<FileCredentialEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file_bytes = std::fs::read(&self.path)
            .map_err(|e| GitPurgeError::Auth(format!("Failed to read secrets file: {}", e)))?;

        if file_bytes.len() < SALT_LEN + NONCE_LEN {
            return Err(GitPurgeError::Auth(
                "Corrupted secrets file: header too short".to_string(),
            ));
        }

        let (salt, rest) = file_bytes.split_at(SALT_LEN);
        let (nonce, ciphertext) = rest.split_at(NONCE_LEN);

        let key = self.derive_key(salt)?;
        let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(key.as_slice());
        let cipher = Aes256Gcm::new(cipher_key);

        let nonce_arr = Nonce::from_slice(nonce);
        let plaintext_bytes = cipher.decrypt(nonce_arr, ciphertext).map_err(|_| {
            GitPurgeError::Auth("Decryption failed: wrong passphrase or corrupted file".to_string())
        })?;

        let entries: Vec<FileCredentialEntry> = serde_json::from_slice(&plaintext_bytes)
            .map_err(|e| GitPurgeError::Auth(format!("Corrupted secrets file data: {}", e)))?;

        Ok(entries)
    }

    /// Encrypt and write the entries to the file.
    fn write_entries(&self, entries: &[FileCredentialEntry]) -> Result<()> {
        let _guard = self.lock.lock().unwrap();

        // Ensure parent directories exist
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                GitPurgeError::Auth(format!(
                    "Failed to create parent directory for secrets: {}",
                    e
                ))
            })?;
        }

        // Generate random salt and nonce
        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; NONCE_LEN];
        let mut rng = thread_rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce);

        // Derive key and encrypt
        let key = self.derive_key(&salt)?;
        let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(key.as_slice());
        let cipher = Aes256Gcm::new(cipher_key);

        let plaintext_bytes = serde_json::to_vec(entries)
            .map_err(|e| GitPurgeError::Auth(format!("Failed to serialize credentials: {}", e)))?;

        let nonce_arr = Nonce::from_slice(&nonce);
        let ciphertext = cipher
            .encrypt(nonce_arr, plaintext_bytes.as_slice())
            .map_err(|e| GitPurgeError::Auth(format!("Encryption failed: {}", e)))?;

        // Construct final payload
        let mut payload = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
        payload.extend_from_slice(&salt);
        payload.extend_from_slice(&nonce);
        payload.extend_from_slice(&ciphertext);

        // Atomic write
        let temp_path = self.path.with_extension("tmp");
        std::fs::write(&temp_path, &payload).map_err(|e| {
            GitPurgeError::Auth(format!("Failed to write temporary secrets file: {}", e))
        })?;

        // Restrict file permissions to owner-only on Unix before renaming
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&temp_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&temp_path, perms);
            }
        }

        std::fs::rename(&temp_path, &self.path)
            .map_err(|e| GitPurgeError::Auth(format!("Failed to save secrets file: {}", e)))?;

        Ok(())
    }
}

impl SecretStore for FileSecretStore {
    fn store(
        &self,
        repo: &RepoId,
        remote: &str,
        kind: CredentialKind,
        secret: &[u8],
    ) -> Result<()> {
        let label = format!("{kind:?} for {remote}");
        let mut entries = self.read_entries()?;

        // Update or insert
        entries.retain(|e| e.repo != *repo || e.remote != remote);
        entries.push(FileCredentialEntry {
            repo: repo.clone(),
            remote: remote.to_string(),
            kind,
            label,
            secret: secret.to_vec(),
        });

        self.write_entries(&entries)?;
        Ok(())
    }

    fn retrieve(&self, repo: &RepoId, remote: &str) -> Result<Option<Credential>> {
        let entries = self.read_entries()?;
        let found = entries
            .into_iter()
            .find(|e| e.repo == *repo && e.remote == remote);

        Ok(found.map(|e| Credential::new(e.kind, e.label, e.secret)))
    }

    fn remove(&self, repo: &RepoId, remote: &str) -> Result<()> {
        let mut entries = self.read_entries()?;
        let original_len = entries.len();
        entries.retain(|e| e.repo != *repo || e.remote != remote);

        if entries.len() != original_len {
            self.write_entries(&entries)?;
        }
        Ok(())
    }

    fn list(&self) -> Result<Vec<CredentialEntry>> {
        let entries = self.read_entries()?;
        Ok(entries
            .into_iter()
            .map(|e| CredentialEntry {
                repo: e.repo,
                remote: e.remote,
                kind: e.kind,
                label: e.label,
            })
            .collect())
    }

    fn test(&self, _repo: &RepoId, _remote: &str) -> Result<bool> {
        // If we can read entries successfully, the passphrase is correct.
        match self.read_entries() {
            Ok(_) => Ok(true),
            Err(GitPurgeError::Auth(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_store_missing_file() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("secrets.enc");
        let store = FileSecretStore::new(file_path, "correct-passphrase");

        // Retrieve and list on missing file should be Ok/None
        let repo = RepoId("missing-repo".to_string());
        let retrieved = store.retrieve(&repo, "origin").unwrap();
        assert!(retrieved.is_none());

        let list = store.list().unwrap();
        assert!(list.is_empty());

        let test_res = store.test(&repo, "origin").unwrap();
        assert!(test_res); // No file is considered valid key because it can be initialized
    }

    #[test]
    fn test_file_store_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("secrets.enc");
        let store = FileSecretStore::new(&file_path, "my-secure-passphrase");

        let repo = RepoId("repo-abc".to_string());
        let remote = "origin";
        let secret = b"my-super-secret-token";

        // Store
        store
            .store(&repo, remote, CredentialKind::HttpsToken, secret)
            .unwrap();

        // Retrieve
        let retrieved = store.retrieve(&repo, remote).unwrap().unwrap();
        assert_eq!(retrieved.secret(), secret);
        assert_eq!(retrieved.kind(), CredentialKind::HttpsToken);
        assert_eq!(retrieved.label(), "HttpsToken for origin");

        // List
        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].repo, repo);
        assert_eq!(list[0].remote, remote);
        assert_eq!(list[0].kind, CredentialKind::HttpsToken);
        assert_eq!(list[0].label, "HttpsToken for origin");

        // Test
        assert!(store.test(&repo, remote).unwrap());

        // Remove
        store.remove(&repo, remote).unwrap();
        let retrieved_after = store.retrieve(&repo, remote).unwrap();
        assert!(retrieved_after.is_none());
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn test_file_store_invalid_passphrase() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("secrets.enc");
        let store1 = FileSecretStore::new(&file_path, "correct-passphrase");
        let store2 = FileSecretStore::new(&file_path, "wrong-passphrase");

        let repo = RepoId("repo-abc".to_string());
        let remote = "origin";
        let secret = b"my-super-secret-token";

        // Store with store1
        store1
            .store(&repo, remote, CredentialKind::HttpsToken, secret)
            .unwrap();

        // Retrieve with store2 should fail
        let retrieve_res = store2.retrieve(&repo, remote);
        assert!(retrieve_res.is_err());
        match retrieve_res {
            Err(GitPurgeError::Auth(msg)) => {
                assert!(msg.contains("Decryption failed"));
            }
            other => panic!("Expected Auth error, got {:?}", other),
        }

        // Test with store2 should return Ok(false)
        assert!(!store2.test(&repo, remote).unwrap());
    }

    #[test]
    fn test_file_store_corrupted_file() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("secrets.enc");
        let store = FileSecretStore::new(&file_path, "passphrase");

        // Write junk data
        std::fs::write(&file_path, b"too-short-junk").unwrap();
        let res = store.retrieve(&RepoId("x".to_string()), "y");
        assert!(res.is_err());

        std::fs::write(&file_path, vec![0u8; 100]).unwrap();
        let res2 = store.retrieve(&RepoId("x".to_string()), "y");
        assert!(res2.is_err());
    }

    #[test]
    fn test_file_store_safe_07_secrecy() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("secrets.enc");
        let store = FileSecretStore::new(&file_path, "passphrase");

        let repo = RepoId("sensitive-repo".to_string());
        let remote = "origin";
        let secret = b"super-secret-clear-text-password";

        store
            .store(&repo, remote, CredentialKind::HttpsBasic, secret)
            .unwrap();

        // Read raw file bytes from disk
        let file_bytes = std::fs::read(&file_path).unwrap();

        // Assert that the raw secret or label/username is NOT present in clear text in the file
        let secret_str = std::str::from_utf8(secret).unwrap();
        let file_contents_lossy = String::from_utf8_lossy(&file_bytes);
        assert!(!file_contents_lossy.contains(secret_str));
        assert!(!file_contents_lossy.contains("sensitive-repo"));
        assert!(!file_contents_lossy.contains("HttpsBasic for origin"));
    }
}
