//! Configuration load/save (CONVENTIONS §5).
//!
//! Config lives at `<config_dir>/git-purge/config.toml`, resolved via the
//! `directories` crate (XDG on Linux, Known Folders on Windows, Standard Dirs on
//! macOS). **No path is ever hardcoded** — the legacy scripts' `/home/mgamil/...`
//! hardcoding is the exact anti-pattern this replaces.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::Policy;

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
}

impl Config {
    /// Resolve the config file path via the `directories` crate.
    ///
    // TODO(P0-T2): use `directories::ProjectDirs::from("com", "gitpurge", "git-purge")`
    // and return `<config_dir>/config.toml`.
    pub fn default_path() -> Result<PathBuf> {
        todo!("resolve config path via directories — phase P0")
    }

    /// Load config from the given path (or the default path when `None`), returning
    /// [`Config::default`] when the file does not yet exist.
    ///
    /// # Errors
    /// Returns [`crate::GitPurgeError::Config`] on parse/validation failure.
    pub fn load(path: Option<&std::path::Path>) -> Result<Self> {
        let _ = path;
        // TODO(P0-T2): read file, `toml::from_str`, validate, merge defaults.
        todo!("config load — phase P0")
    }

    /// Persist config as TOML to the given path (or the default path when `None`).
    ///
    /// # Errors
    /// Returns [`crate::GitPurgeError::Config`] / IO errors on failure.
    pub fn save(&self, path: Option<&std::path::Path>) -> Result<()> {
        let _ = path;
        // TODO(P0-T2): `toml::to_string_pretty`, create parent dirs, atomic write.
        todo!("config save — phase P0")
    }
}
