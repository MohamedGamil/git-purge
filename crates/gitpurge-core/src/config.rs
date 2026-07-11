//! Configuration load/save (CONVENTIONS §5).
//!
//! Config lives at `<config_dir>/git-purge/config.toml`, resolved via the
//! `directories` crate (XDG on Linux, Known Folders on Windows, Standard Dirs on
//! macOS). **No path is ever hardcoded** — the legacy scripts' `/home/mgamil/...`
//! hardcoding is the exact anti-pattern this replaces.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::{Policy, RepoId, Repository};

/// Top-level user configuration, deserialized from `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Root directory under which per-repo bare mirrors and snapshots live.
    /// `None` => the resolved default `<data_dir>/git-purge/backups/`.
    pub backups_root: Option<PathBuf>,

    /// The default policy (age threshold, naming rules, protection list) applied to
    /// every repo unless overridden per-repo or via CLI/UI flags.
    pub default_policy: Policy,

    /// Extra user-supplied protected branch names, unioned with the immutable
    /// well-known set (`main`, `master`, `develop`, `staging`, `production`, `HEAD`).
    pub protected: Vec<String>,

    /// Tracked repositories managed by Git Purge.
    pub repos: Vec<Repository>,

    /// The default repository ID to use when `--repo` is not supplied.
    pub default_repo: Option<RepoId>,
}

impl Config {
    /// Resolve the config file path via the `directories` crate.
    pub fn default_path() -> Result<PathBuf> {
        let proj_dirs =
            directories::ProjectDirs::from("com", "gitpurge", "git-purge").ok_or_else(|| {
                crate::GitPurgeError::Config("Could not resolve home/project directory".to_string())
            })?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    /// Load config from the given path (or the default path when `None`), returning
    /// [`Config::default`] when the file does not yet exist.
    ///
    /// # Errors
    /// Returns [`crate::GitPurgeError::Config`] on parse/validation failure.
    pub fn load(path: Option<&std::path::Path>) -> Result<Self> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_path()?,
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to read config: {}", e)))?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to parse config TOML: {}", e))
        })?;

        Ok(config)
    }

    /// Persist config as TOML to the given path (or the default path when `None`).
    ///
    /// # Errors
    /// Returns [`crate::GitPurgeError::Config`] / IO errors on failure.
    pub fn save(&self, path: Option<&std::path::Path>) -> Result<()> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_path()?,
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to create config dir: {}", e))
            })?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(&path, content).map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_roundtrip() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path();

        let mut config = Config::default();
        config.protected.push("main-legacy".to_string());

        config.save(Some(path)).unwrap();

        let loaded = Config::load(Some(path)).unwrap();
        assert_eq!(loaded.protected, vec!["main-legacy".to_string()]);
    }
}
