//! Credential model types (P11-T1).
//!
//! **SAFE-07**: secret-bearing types redact all material in `Debug` output.

use crate::model::RepoId;

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

/// Metadata-only credential reference returned by [`super::SecretStore::list`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CredentialEntry {
    /// Owning repository id.
    pub repo: RepoId,
    /// Remote name (e.g. `origin`).
    pub remote: String,
    /// Credential kind.
    pub kind: CredentialKind,
    /// Display label — never the secret.
    pub label: String,
}

/// Lookup key for credential resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialQuery {
    /// Repository to resolve credentials for.
    pub repo: RepoId,
    /// Remote name to resolve credentials for.
    pub remote: String,
    /// Optional kind filter when multiple credentials could match.
    pub kind: Option<CredentialKind>,
}

impl CredentialQuery {
    /// Create a lookup for a repo + remote pair.
    pub fn new(repo: RepoId, remote: impl Into<String>) -> Self {
        Self {
            repo,
            remote: remote.into(),
            kind: None,
        }
    }

    /// Restrict resolution to a specific credential kind.
    pub fn with_kind(mut self, kind: CredentialKind) -> Self {
        self.kind = Some(kind);
        self
    }
}

/// A credential with secret material (retrieve result).
pub struct Credential {
    kind: CredentialKind,
    label: String,
    secret: zeroize::Zeroizing<Vec<u8>>,
}

impl Credential {
    /// Build a credential from kind, label, and secret bytes.
    pub fn new(kind: CredentialKind, label: impl Into<String>, secret: impl Into<Vec<u8>>) -> Self {
        Self {
            kind,
            label: label.into(),
            secret: zeroize::Zeroizing::new(secret.into()),
        }
    }

    /// Credential kind.
    pub fn kind(&self) -> CredentialKind {
        self.kind
    }

    /// Display label (never contains the secret).
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Secret bytes for authentication. Handle with care — never log or print.
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("kind", &self.kind)
            .field("label", &self.label)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

impl Clone for Credential {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            label: self.label.clone(),
            secret: zeroize::Zeroizing::new(self.secret.to_vec()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_debug_never_leaks_token() {
        let token = "super-secret-token-12345";
        let cred = Credential::new(
            CredentialKind::HttpsToken,
            "GitHub token for origin",
            token.as_bytes(),
        );

        let debug = format!("{cred:?}");
        assert!(!debug.contains(token));
        assert!(debug.contains("[REDACTED]"));
        assert_eq!(cred.secret(), token.as_bytes());
    }

    #[test]
    fn credential_kind_variants_covered() {
        let kinds = [
            CredentialKind::SshKey,
            CredentialKind::HttpsBasic,
            CredentialKind::HttpsToken,
            CredentialKind::SshAgent,
        ];
        assert_eq!(kinds.len(), 4);
    }

    #[test]
    fn credential_query_builder() {
        let query = CredentialQuery::new(RepoId("repo".into()), "origin")
            .with_kind(CredentialKind::HttpsToken);
        assert_eq!(query.remote, "origin");
        assert_eq!(query.kind, Some(CredentialKind::HttpsToken));
    }
}
