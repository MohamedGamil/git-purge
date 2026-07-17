//! Core config & repository methods (P10-T2).

use crate::config::Config;
use crate::error::Result;
use crate::model::{RepoId, Repository};
use std::path::Path;

impl super::Engine {
    /// Register a repository in the local in-memory store.
    pub fn register_repo(&self, repo: Repository) -> Result<()> {
        self.history.save_repo(&repo)?;
        let mut repos = self.repos.lock().unwrap();
        repos.insert(repo.id.clone(), repo);
        Ok(())
    }

    /// Add a repository to the tracked list in config and save.
    pub fn add_repo(&self, repo: Repository) -> Result<()> {
        self.register_repo(repo.clone())?;
        // Sync config
        let mut config = self.config.lock().unwrap();
        if !config.repos.iter().any(|r| r.id == repo.id) {
            config.repos.push(repo);
        } else {
            // Update existing
            if let Some(existing) = config.repos.iter_mut().find(|r| r.id == repo.id) {
                *existing = repo;
            }
        }
        Ok(())
    }

    /// Remove a repository from the tracked list.
    pub fn remove_repo(&self, id: &RepoId) -> Result<()> {
        {
            let mut repos = self.repos.lock().unwrap();
            repos.remove(id);
        }
        let mut config = self.config.lock().unwrap();
        config.repos.retain(|r| r.id != *id);
        if config.default_repo.as_ref() == Some(id) {
            config.default_repo = None;
        }
        Ok(())
    }

    /// List all tracked repositories.
    pub fn list_repos(&self) -> Result<Vec<Repository>> {
        let repos = self.repos.lock().unwrap();
        Ok(repos.values().cloned().collect())
    }

    /// Get tracked repository details by ID.
    pub fn get_repo(&self, id: &RepoId) -> Result<Option<Repository>> {
        let repos = self.repos.lock().unwrap();
        Ok(repos.get(id).cloned())
    }

    /// Get all configured remotes for a repository.
    pub fn get_remotes(&self, id: &RepoId) -> Result<Vec<String>> {
        let repo_model = {
            let repos = self.repos.lock().unwrap();
            repos.get(id).cloned().ok_or_else(|| {
                crate::GitPurgeError::RepoNotFound(format!("Repository not registered: {:?}", id))
            })?
        };
        let local_path = repo_model.local_path.as_ref().ok_or_else(|| {
            crate::GitPurgeError::RepoNotFound("Local path missing for repository".to_string())
        })?;
        let git2_repo = git2::Repository::open(local_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open repository: {}", e)))?;
        let remotes_list = git2_repo
            .remotes()
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to list remotes: {}", e)))?;
        Ok(remotes_list
            .iter()
            .flatten()
            .map(|s| s.to_string())
            .collect())
    }

    /// Set the default repository.
    pub fn set_default_repo(&self, id: RepoId) -> Result<()> {
        let mut config = self.config.lock().unwrap();
        config.default_repo = Some(id);
        Ok(())
    }

    /// Get the default repository ID.
    pub fn default_repo_id(&self) -> Option<RepoId> {
        let config = self.config.lock().unwrap();
        config.default_repo.clone()
    }

    /// Save the current config to disk.
    pub fn save_config(&self, path: Option<&Path>) -> Result<()> {
        let config = self.config.lock().unwrap().clone();
        config.save(path)?;
        Ok(())
    }

    /// Get a clone of the current configuration.
    pub fn config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    /// Update the current configuration.
    pub fn update_config(&self, new_config: Config) {
        let mut config = self.config.lock().unwrap();
        *config = new_config;
    }
}
