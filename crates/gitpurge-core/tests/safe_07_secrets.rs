//! safe_07 secret hygiene regression suite (P11-T6).
//!
//! Enforces that no secret material ever appears in logs, debug outputs,
//! or report listings under SAFE-07 safety invariants.

use gitpurge_core::auth::{Credential, CredentialKind, KeyringSecretStore, SecretStore};
use gitpurge_core::model::RepoId;

#[test]
fn test_safe_07_credential_debug_redacted() {
    let secret = b"my-super-secret-ssh-key-data-12345";
    let cred = Credential::new(
        CredentialKind::SshKey,
        "test-ssh-key-label".to_string(),
        secret.to_vec(),
    );

    // Verify debug output does not leak the secret bytes
    let debug_str = format!("{:?}", cred);
    assert!(
        !debug_str.contains("my-super-secret-ssh-key-data-12345"),
        "Secret leaked in Credential debug representation: {}",
        debug_str
    );
    assert!(
        debug_str.contains("[REDACTED]"),
        "Credential debug should print REDACTED placeholder"
    );
}

#[test]
fn test_safe_07_keyring_store_never_leaks_secrets() {
    let store = KeyringSecretStore::new("git-purge-safety-test");
    let repo = RepoId("safety-repo".to_string());
    let remote = "origin";
    let secret_bytes = b"super-sensitive-api-token-9911";

    // Store the credential
    let _ = store.store(&repo, remote, CredentialKind::HttpsToken, secret_bytes);

    // 1. Verify SecretStore Debug implementation does not leak secrets
    let debug_str = format!("{:?}", store);
    assert!(
        !debug_str.contains("super-sensitive-api-token-9911"),
        "Secret leaked in KeyringSecretStore debug: {}",
        debug_str
    );

    // 2. Verify list output (metadata only) does not contain secret bytes
    if let Ok(list) = store.list() {
        for entry in list {
            let entry_dbg = format!("{:?}", entry);
            assert!(
                !entry_dbg.contains("super-sensitive-api-token-9911"),
                "Secret leaked in CredentialEntry debug: {}",
                entry_dbg
            );
        }
    }

    // Clean up
    let _ = store.remove(&repo, remote);
}
