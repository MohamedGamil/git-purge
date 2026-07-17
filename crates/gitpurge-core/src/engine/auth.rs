//! Auth methods for Engine (P10-T2 & P11-T7).

use crate::auth;
use crate::error::Result;
use crate::model::RepoId;

impl super::Engine {
    /// Store a credential and update configuration metadata.
    pub fn auth_store(
        &self,
        repo: &RepoId,
        remote: &str,
        kind: auth::CredentialKind,
        secret: &[u8],
        metadata: Option<crate::config::CredentialMetadata>,
    ) -> Result<()> {
        self.secrets.store(repo, remote, kind, secret)?;

        if let Some(meta) = metadata {
            let mut config = self.config.lock().unwrap();
            let auth_config = config.auth.get_or_insert_with(Default::default);
            auth_config.credentials.retain(|m| m.id != meta.id);
            auth_config.credentials.push(meta);
        }
        Ok(())
    }

    /// Retrieve a credential.
    pub fn auth_retrieve(&self, repo: &RepoId, remote: &str) -> Result<Option<auth::Credential>> {
        self.secrets.retrieve(repo, remote)
    }

    /// Remove a credential and update configuration metadata.
    pub fn auth_remove(&self, repo: &RepoId, remote: &str) -> Result<()> {
        self.secrets.remove(repo, remote)?;

        let mut config = self.config.lock().unwrap();
        if let Some(ref mut auth_config) = config.auth {
            auth_config.credentials.retain(|m| m.id != repo.0);
        }
        Ok(())
    }

    /// List all credentials (metadata only).
    pub fn auth_list(&self) -> Result<Vec<auth::CredentialEntry>> {
        self.secrets.list()
    }

    /// Test a credential.
    pub fn auth_test(&self, repo: &RepoId, remote: &str) -> Result<bool> {
        self.secrets.test(repo, remote)
    }
}
