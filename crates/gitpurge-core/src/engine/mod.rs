//! Engine facade (P10-T2).

use crate::config::Config;
use crate::error::Result;
use crate::git::GitBackend;
use crate::model::{RepoId, Repository, ScanResult};
use std::collections::HashMap;
use std::sync::Mutex;

pub mod auth;
pub mod backup;
pub mod core;
pub mod execute;
pub mod git_ops;
pub mod report_history;
pub mod scan_plan;

#[cfg(test)]
mod tests;

/// The public facade over `gitpurge-core`.
#[derive(Debug)]
pub struct Engine {
    pub(crate) config: Mutex<Config>,
    pub(crate) git: Box<dyn crate::git::GitBackend>,
    #[allow(dead_code)]
    pub(crate) secrets: Box<dyn crate::auth::SecretStore>,
    #[allow(dead_code)]
    pub(crate) history: Box<dyn crate::history::HistoryStore>,
    #[allow(dead_code)]
    pub(crate) report_sink: Box<dyn crate::report::ReportSink>,
    pub(crate) clock: Box<dyn crate::clock::Clock>,
    #[allow(dead_code)]
    pub(crate) progress: Box<dyn crate::progress::ProgressSink>,
    pub(crate) repos: Mutex<HashMap<RepoId, Repository>>,
    pub(crate) scan_cache: Mutex<HashMap<RepoId, (String, ScanResult)>>,
}

const _: () = {
    fn _assert_send_sync<T: Send + Sync>() {}
    fn _check() {
        _assert_send_sync::<Engine>();
    }
};

impl Engine {
    /// Open an engine with the given configuration, wiring up the default production
    /// adapters (gix/git2 git backend, keyring secret store, SQLite history store).
    #[allow(unsafe_code)]
    pub fn open(config: Config) -> Result<Self> {
        let data_dir = config.resolve_data_dir();
        if let Some(bd) = directories::BaseDirs::new() {
            let old_dir = bd.home_dir().join(".git-purge");
            if old_dir.exists() && !data_dir.exists() {
                let _ = std::fs::rename(&old_dir, &data_dir);
            }
        }

        // Configure libgit2 network connection and operation timeouts to protect against offline state/VPN loss.
        unsafe {
            let _ = git2::opts::set_server_connect_timeout_in_milliseconds(5000);
            let _ = git2::opts::set_server_timeout_in_milliseconds(15000);
        }

        let git = Box::new(crate::git::CompositeGitBackend::new());
        let keyring_store = std::sync::Arc::new(crate::auth::KeyringSecretStore::default());
        let passphrase = std::env::var("GIT_PURGE_PASSPHRASE")
            .unwrap_or_else(|_| "default_git_purge_passphrase".to_string());
        let file_store_path = config.resolve_data_dir().join("vault.json");
        let file_store = std::sync::Arc::new(crate::auth::FileSecretStore::new(
            file_store_path,
            passphrase,
        ));

        let resolver = std::sync::Arc::new(
            crate::auth::CredentialResolver::new()
                .with_config(config.clone())
                .with_keyring_store(keyring_store)
                .with_file_store(file_store),
        );

        git.set_resolver(resolver);

        let secrets = Box::new(crate::auth::FallbackSecretStore::new(
            crate::auth::KeyringSecretStore::default(),
            crate::auth::FileSecretStore::new(
                config.resolve_data_dir().join("vault.json"),
                std::env::var("GIT_PURGE_PASSPHRASE")
                    .unwrap_or_else(|_| "default_git_purge_passphrase".to_string()),
            ),
        ));
        let db_path = config.resolve_data_dir().join("history.db");
        let backups_root = config
            .backups_root
            .clone()
            .unwrap_or_else(|| config.resolve_data_dir().join("backups"));
        let history = Box::new(crate::history::SqliteHistoryStore::new(
            &db_path,
            backups_root.clone(),
        )?);

        let backups_root_clone = backups_root.clone();
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let stmt = conn
                .prepare("SELECT id, repo_id FROM snapshots WHERE backup_path IS NULL;")
                .ok();
            if let Some(mut stmt) = stmt {
                if let Ok(rows_iter) = stmt.query_map([], |row| {
                    let id: String = row.get(0)?;
                    let repo_id: String = row.get(1)?;
                    Ok((id, repo_id))
                }) {
                    let rows: Vec<(String, String)> = rows_iter.filter_map(|r| r.ok()).collect();
                    for (id, repo_id) in rows {
                        let sanitized_id: String = repo_id
                            .chars()
                            .map(|c| {
                                if c.is_alphanumeric() || c == '-' || c == '_' {
                                    c
                                } else {
                                    '_'
                                }
                            })
                            .collect();

                        let target_path = backups_root_clone.join(format!("{}.git", sanitized_id));
                        let resolved_path = if target_path.exists() {
                            Some(target_path)
                        } else {
                            let old_path = backups_root_clone
                                .join("backups")
                                .join(format!("{}.git", sanitized_id));
                            if old_path.exists() {
                                Some(old_path)
                            } else if let Some(parent) = backups_root_clone.parent() {
                                let other_path = parent.join(format!("{}.git", sanitized_id));
                                if other_path.exists() {
                                    Some(other_path)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };

                        if let Some(path) = resolved_path {
                            let _ = conn.execute(
                                "UPDATE snapshots SET backup_path = ?1 WHERE id = ?2;",
                                (path.to_string_lossy().to_string(), &id),
                            );
                        }
                    }
                }
            }
        }
        let report_sink = Box::new(crate::report::FileReportSink::new(
            config.resolve_data_dir(),
            None,
        ));
        let clock = Box::new(crate::clock::SystemClock);
        let progress = Box::new(crate::progress::NoopProgressSink);

        let repos_map = config
            .repos
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();

        Ok(Self {
            config: Mutex::new(config),
            git,
            secrets,
            history,
            report_sink,
            clock,
            progress,
            repos: Mutex::new(repos_map),
            scan_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Construct a new Engine with custom injected ports (useful for tests).
    pub fn new(
        config: Config,
        git: Box<dyn crate::git::GitBackend>,
        secrets: Box<dyn crate::auth::SecretStore>,
        history: Box<dyn crate::history::HistoryStore>,
        report_sink: Box<dyn crate::report::ReportSink>,
        clock: Box<dyn crate::clock::Clock>,
        progress: Box<dyn crate::progress::ProgressSink>,
    ) -> Self {
        let repos_map = config
            .repos
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();

        Self {
            config: Mutex::new(config),
            git,
            secrets,
            history,
            report_sink,
            clock,
            progress,
            repos: Mutex::new(repos_map),
            scan_cache: Mutex::new(HashMap::new()),
        }
    }
}
