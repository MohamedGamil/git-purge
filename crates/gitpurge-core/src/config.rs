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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Override the data root directory (history DB, backups, reports).
    pub data_dir: Option<PathBuf>,

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

    /// Custom date-time display format.
    pub date_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: None,
            backups_root: None,
            default_policy: Policy::default(),
            protected: Vec::new(),
            repos: Vec::new(),
            default_repo: None,
            date_format: "YYYY-MM-DD h:m a".to_string(),
        }
    }
}

impl Config {
    /// Resolve the data root directory (CONVENTIONS §5).
    pub fn resolve_data_dir(&self) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| {
            directories::BaseDirs::new()
                .map(|base| base.home_dir().join(".gitpurge"))
                .expect("Failed to resolve home directory")
        })
    }

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
        assert_eq!(loaded.date_format, "YYYY-MM-DD h:m a");
    }

    #[test]
    fn test_older_config_deserialization_defaults() {
        let older_toml = r#"
            protected = ["legacy-branch"]
        "#;
        let loaded: Config = toml::from_str(older_toml).unwrap();
        assert_eq!(loaded.date_format, "YYYY-MM-DD h:m a");
        assert_eq!(loaded.protected, vec!["legacy-branch".to_string()]);
    }

    #[test]
    fn test_resolve_data_dir_default() {
        let config = Config::default();
        let data_dir = config.resolve_data_dir();
        let expected = directories::BaseDirs::new()
            .map(|base| base.home_dir().join(".gitpurge"))
            .unwrap();
        assert_eq!(data_dir, expected);
    }
}
