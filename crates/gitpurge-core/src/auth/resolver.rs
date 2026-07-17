//! Credential resolver chain (P11-T4).

use crate::auth::{Credential, CredentialKind, SecretStore};
use crate::config::Config;
use crate::error::{GitPurgeError, Result};
use crate::model::RepoId;
use std::sync::Arc;

/// A resolver that tries credential sources in priority order.
pub struct CredentialResolver {
    config: Option<Config>,
    keyring_store: Option<Arc<dyn SecretStore>>,
    file_store: Option<Arc<dyn SecretStore>>,
    cli_credential: Option<Credential>,
    #[allow(clippy::type_complexity)]
    prompt_fn: Option<Box<dyn Fn(CredentialKind, &str) -> Result<Vec<u8>> + Send + Sync>>,
}

impl std::fmt::Debug for CredentialResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialResolver")
            .field("config", &self.config)
            .field("keyring_store", &self.keyring_store)
            .field("file_store", &self.file_store)
            .field("cli_credential", &self.cli_credential)
            .field("prompt_fn", &self.prompt_fn.as_ref().map(|_| "Some(Fn)"))
            .finish()
    }
}

impl CredentialResolver {
    /// Create a new resolver builder.
    pub fn new() -> Self {
        Self {
            config: None,
            keyring_store: None,
            file_store: None,
            cli_credential: None,
            prompt_fn: None,
        }
    }

    /// Set the configuration.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the keyring secret store.
    pub fn with_keyring_store(mut self, store: Arc<dyn SecretStore>) -> Self {
        self.keyring_store = Some(store);
        self
    }

    /// Set the fallback file secret store.
    pub fn with_file_store(mut self, store: Arc<dyn SecretStore>) -> Self {
        self.file_store = Some(store);
        self
    }

    /// Set the CLI override credential.
    pub fn with_cli_credential(mut self, cred: Credential) -> Self {
        self.cli_credential = Some(cred);
        self
    }

    /// Set the prompt callback.
    pub fn with_prompt(
        mut self,
        prompt: impl Fn(CredentialKind, &str) -> Result<Vec<u8>> + Send + Sync + 'static,
    ) -> Self {
        self.prompt_fn = Some(Box::new(prompt));
        self
    }

    /// Resolve a credential for the given repo + remote.
    pub fn resolve(
        &self,
        repo: &RepoId,
        remote: &str,
        remote_url: &str,
        preferred_kind: Option<CredentialKind>,
    ) -> Result<Credential> {
        // 1. CLI flag override
        if let Some(ref cred) = self.cli_credential {
            return Ok(Credential::new(
                cred.kind(),
                cred.label(),
                cred.secret().to_vec(),
            ));
        }

        // Check for explicit config match in config.toml
        let mut matched_meta = None;
        if let Some(ref config) = self.config {
            if let Some(ref auth) = config.auth {
                let (scheme, host, path) = normalize_url(remote_url);
                let mut best_score = 0;
                let mut ambiguous = false;

                for meta in &auth.credentials {
                    let score = match_specificity(&meta.r#match, &scheme, &host, &path, remote_url);
                    if score > 0 {
                        if score > best_score {
                            best_score = score;
                            matched_meta = Some(meta.clone());
                            ambiguous = false;
                        } else if score == best_score {
                            ambiguous = true;
                        }
                    }
                }

                if ambiguous {
                    return Err(GitPurgeError::Config(
                        "Ambiguous credential match: multiple matching credentials with equal specificity".to_string()
                    ));
                }
            }
        }

        // 2. OS Keyring Store
        let keyring_cred = if let Some(ref store) = self.keyring_store {
            store.retrieve(repo, remote).ok().flatten()
        } else {
            None
        };

        // 3. Fallback File Store
        let file_cred = if keyring_cred.is_none() {
            if let Some(ref store) = self.file_store {
                store.retrieve(repo, remote).ok().flatten()
            } else {
                None
            }
        } else {
            None
        };

        let resolved_cred = keyring_cred.or(file_cred);

        // If we matched config metadata, we construct/adjust the credential
        if let Some(meta) = matched_meta {
            let kind = match meta.method.as_str() {
                "ssh-key" => CredentialKind::SshKey,
                "https-basic" => CredentialKind::HttpsBasic,
                "token" => CredentialKind::HttpsToken,
                "ssh-agent" => CredentialKind::SshAgent,
                _ => CredentialKind::HttpsToken,
            };

            let secret = if let Some(ref cred) = resolved_cred {
                cred.secret().to_vec()
            } else {
                Vec::new()
            };

            let mut cred = Credential::new(kind, meta.id, secret);
            if let Some(kp) = meta.key_path {
                cred = cred.with_key_path(kp);
            }
            return Ok(cred);
        }

        // If no explicit config match, fallback to resolved store credential directly
        if let Some(cred) = resolved_cred {
            return Ok(cred);
        }

        // 4. Environment Variables
        if let Ok(token) = std::env::var("GIT_PURGE_TOKEN") {
            return Ok(Credential::new(
                CredentialKind::HttpsToken,
                "Env: GIT_PURGE_TOKEN",
                token.into_bytes(),
            ));
        }
        if let Ok(password) = std::env::var("GIT_PURGE_PASSWORD") {
            let username =
                std::env::var("GIT_PURGE_USERNAME").unwrap_or_else(|_| "git".to_string());
            return Ok(Credential::new(
                CredentialKind::HttpsBasic,
                format!("Env basic: {}", username),
                password.into_bytes(),
            ));
        }
        if let Ok(ssh_key) = std::env::var("GIT_PURGE_SSH_KEY") {
            return Ok(Credential::new(
                CredentialKind::SshKey,
                "Env: GIT_PURGE_SSH_KEY",
                ssh_key.into_bytes(),
            ));
        }

        // 5. Prompt callback (interactive only)
        if let Some(ref prompt) = self.prompt_fn {
            let kind = preferred_kind.unwrap_or(CredentialKind::HttpsToken);
            let prompt_msg = format!("Enter credentials for repo {} remote {}", repo.0, remote);
            let secret = prompt(kind, &prompt_msg)?;
            return Ok(Credential::new(
                kind,
                format!("Prompted {:?}", kind),
                secret,
            ));
        }

        Err(GitPurgeError::Auth(format!(
            "No credential found for repo {} remote {}",
            repo.0, remote
        )))
    }
}

impl Default for CredentialResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to normalize URLs for matching.
fn normalize_url(url: &str) -> (String, String, String) {
    let url_trimmed = url.trim().trim_end_matches(".git");

    if url_trimmed.contains('@')
        && url_trimmed.contains(':')
        && !url_trimmed.starts_with("http")
        && !url_trimmed.starts_with("ssh://")
    {
        let parts: Vec<&str> = url_trimmed.split(':').collect();
        if parts.len() >= 2 {
            let host_part = parts[0];
            let path = parts[1..].join(":");
            let host = host_part.split('@').next_back().unwrap_or(host_part);
            return ("ssh".to_string(), host.to_lowercase(), path);
        }
    }

    let mut scheme = "https".to_string();
    let mut host_and_path = url_trimmed;
    if let Some(pos) = url_trimmed.find("://") {
        scheme = url_trimmed[..pos].to_lowercase();
        host_and_path = &url_trimmed[pos + 3..];
    }

    let parts: Vec<&str> = host_and_path.splitn(2, '/').collect();
    let host_with_port = parts[0];
    let path = if parts.len() > 1 { parts[1] } else { "" };

    let host = host_with_port
        .split(':')
        .next()
        .unwrap_or(host_with_port)
        .to_lowercase();
    (scheme, host, path.to_string())
}

/// Compute specificity score for a match spec against normalized URL.
fn match_specificity(
    match_spec: &str,
    scheme: &str,
    host: &str,
    path: &str,
    original_url: &str,
) -> usize {
    let spec = match_spec.trim().trim_end_matches(".git");

    if spec == original_url || spec == format!("{}://{}/{}", scheme, host, path) {
        return 3;
    }

    if spec.contains('/') {
        let parts: Vec<&str> = spec.splitn(2, '/').collect();
        let spec_host = parts[0].to_lowercase();
        let spec_path = parts[1];

        if spec_host == host {
            if let Some(prefix) = spec_path.strip_suffix('*') {
                if path.starts_with(prefix) {
                    return 2;
                }
            } else if spec_path == path {
                return 2;
            }
        }
    }

    if spec.to_lowercase() == host {
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::FakeSecretStore;
    use crate::config::AuthConfig;
    use crate::config::CredentialMetadata;

    #[test]
    fn test_resolver_priority() {
        let repo = RepoId("test-repo".to_string());
        let remote = "origin";
        let remote_url = "https://github.com/myorg/myrepo";

        // Setup stores
        let keyring = Arc::new(FakeSecretStore::default());
        let file = Arc::new(FakeSecretStore::default());

        keyring
            .store(&repo, remote, CredentialKind::HttpsToken, b"keyring-secret")
            .unwrap();
        file.store(&repo, remote, CredentialKind::HttpsToken, b"file-secret")
            .unwrap();

        // 1. CLI flag priority
        let resolver = CredentialResolver::new()
            .with_keyring_store(keyring.clone())
            .with_file_store(file.clone())
            .with_cli_credential(Credential::new(
                CredentialKind::HttpsToken,
                "cli",
                b"cli-secret",
            ));
        let resolved = resolver.resolve(&repo, remote, remote_url, None).unwrap();
        assert_eq!(resolved.secret(), b"cli-secret");

        // 2. Keyring priority (no CLI flag)
        let resolver = CredentialResolver::new()
            .with_keyring_store(keyring.clone())
            .with_file_store(file.clone());
        let resolved = resolver.resolve(&repo, remote, remote_url, None).unwrap();
        assert_eq!(resolved.secret(), b"keyring-secret");

        // 3. File fallback priority (no keyring, no CLI flag)
        let resolver = CredentialResolver::new().with_file_store(file.clone());
        let resolved = resolver.resolve(&repo, remote, remote_url, None).unwrap();
        assert_eq!(resolved.secret(), b"file-secret");

        // 4. Env var priority
        std::env::set_var("GIT_PURGE_TOKEN", "env-secret");
        let resolver = CredentialResolver::new();
        let resolved = resolver.resolve(&repo, remote, remote_url, None).unwrap();
        assert_eq!(resolved.secret(), b"env-secret");
        std::env::remove_var("GIT_PURGE_TOKEN");

        // 5. Prompt priority
        let resolver =
            CredentialResolver::new().with_prompt(|_kind, _msg| Ok(b"prompt-secret".to_vec()));
        let resolved = resolver.resolve(&repo, remote, remote_url, None).unwrap();
        assert_eq!(resolved.secret(), b"prompt-secret");
    }

    #[test]
    fn test_resolver_explicit_config_match() {
        let repo = RepoId("test-repo".to_string());
        let remote = "origin";
        let remote_url = "git@github.com:acme/api.git";

        let config = Config {
            auth: Some(AuthConfig {
                credentials: vec![CredentialMetadata {
                    id: "github-ssh".to_string(),
                    method: "ssh-key".to_string(),
                    r#match: "github.com".to_string(),
                    username: Some("git".to_string()),
                    key_path: Some(std::path::PathBuf::from("~/.ssh/id_ed25519")),
                }],
            }),
            ..Default::default()
        };

        let keyring = Arc::new(FakeSecretStore::default());
        // Store passphrase in store
        keyring
            .store(&repo, remote, CredentialKind::SshKey, b"passphrase123")
            .unwrap();

        let resolver = CredentialResolver::new()
            .with_config(config)
            .with_keyring_store(keyring);

        let resolved = resolver.resolve(&repo, remote, remote_url, None).unwrap();
        assert_eq!(resolved.kind(), CredentialKind::SshKey);
        assert_eq!(resolved.label(), "github-ssh");
        assert_eq!(resolved.secret(), b"passphrase123");
        assert_eq!(
            resolved.key_path().unwrap().to_str().unwrap(),
            "~/.ssh/id_ed25519"
        );
    }
}
