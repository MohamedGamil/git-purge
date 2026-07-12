//! SQLite history store implementation (CONVENTIONS §5, doc 10 §3)

use rusqlite::{Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::Result;
use crate::history::HistoryStore;
use crate::model::{
    RepoId, Repository, RunRecord, RunReport, Snapshot, SnapshotId, SnapshotTrigger, TrendEntry,
    TrendHistory,
};

/// SQLite adapter for history and trend tracking.
#[derive(Debug)]
pub struct SqliteHistoryStore {
    conn: Mutex<Connection>,
    backups_root: PathBuf,
}

impl SqliteHistoryStore {
    /// Open or create a SQLite database at the given path, running migrations.
    pub fn new(db_path: &Path, backups_root: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to create DB directory: {}", e))
            })?;
        }
        let mut conn = Connection::open(db_path)
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to open database: {}", e)))?;

        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to enable foreign keys: {}", e))
            })?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to enable WAL: {}", e)))?;

        super::migrate::migrate(&mut conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            backups_root,
        })
    }

    fn resolve_mirror_path(&self, repo_id: &RepoId) -> PathBuf {
        let sanitized_id: String = repo_id
            .0
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.backups_root.join(format!("{}.git", sanitized_id))
    }

    fn load_snapshot_from_mirror(
        &self,
        repo_id: &RepoId,
        snapshot_id: &SnapshotId,
    ) -> Result<Snapshot> {
        let mirror_path = self.resolve_mirror_path(repo_id);
        if !mirror_path.exists() {
            return Err(crate::GitPurgeError::Snapshot(format!(
                "Bare mirror not found: {:?}",
                mirror_path
            )));
        }
        let mirror_repo = git2::Repository::open_bare(&mirror_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open bare mirror: {}", e)))?;
        let meta_ref_path = format!("refs/gitpurge/meta/{}", snapshot_id.0);
        let meta_ref = mirror_repo.find_reference(&meta_ref_path).map_err(|e| {
            crate::GitPurgeError::Snapshot(format!("Snapshot meta ref not found: {}", e))
        })?;
        let oid = meta_ref.target().ok_or_else(|| {
            crate::GitPurgeError::Snapshot("Meta reference points to no OID".to_string())
        })?;
        let blob = mirror_repo.find_blob(oid).map_err(|e| {
            crate::GitPurgeError::Snapshot(format!("Failed to find metadata blob: {}", e))
        })?;
        let snapshot: Snapshot = serde_json::from_slice(blob.content()).map_err(|e| {
            crate::GitPurgeError::Config(format!("Failed to parse manifest: {}", e))
        })?;
        Ok(snapshot)
    }
}

impl HistoryStore for SqliteHistoryStore {
    fn save_repo(&self, repo: &Repository) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let canonical_url = repo.remote_url.as_ref().map(|u| u.raw.as_str());
        let local_path = repo.local_path.as_ref().map(|p| p.to_str().unwrap_or(""));

        // compute path_hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        canonical_url.unwrap_or("").hash(&mut hasher);
        local_path.unwrap_or("").hash(&mut hasher);
        let path_hash = format!("{:x}", hasher.finish());

        let default_branch = repo
            .default_branch
            .as_ref()
            .map(|b| b.name.0.clone())
            .unwrap_or_else(|| "origin/main".to_string());
        let created_at = repo
            .added_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();

        conn.execute(
            "INSERT OR REPLACE INTO repos (id, canonical_url, local_path, path_hash, display_name, default_branch, created_at, tombstoned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL);",
            (
                &repo.id.0,
                canonical_url,
                local_path,
                path_hash,
                &repo.display_name,
                default_branch,
                created_at,
            ),
        ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to save repository: {}", e)))?;
        Ok(())
    }

    fn record_run(&self, report: &RunReport) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let finished_at = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        let started_at_str = report
            .started_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        let actor = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "system".to_string());
        let mode_str = match report.mode {
            crate::model::ExecMode::DryRun => "dry-run",
            crate::model::ExecMode::Execute => "execute",
        };

        // 1. Insert into runs
        conn.execute(
            "INSERT INTO runs (id, repo_id, command, mode, started_at, finished_at, snapshot_id, age_threshold, actor, tool_version, exit_code, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9, NULL, NULL)
             ON CONFLICT(repo_id, started_at, command) DO UPDATE SET finished_at = excluded.finished_at;",
            (
                &report.id,
                &report.repo.0,
                &report.command,
                mode_str,
                &started_at_str,
                &finished_at,
                report.snapshot.as_ref().map(|s| &s.0),
                &actor,
                env!("CARGO_PKG_VERSION"),
            ),
        ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to insert run: {}", e)))?;

        // 2. Insert into metrics (dedup logic)
        if let Some(metrics) = &report.metrics {
            let hash_str = format!(
                "{}:{}:{}:{}:{}:{}",
                metrics.total,
                metrics.active,
                metrics.stale,
                metrics.merged,
                metrics.unmerged,
                metrics.non_standard
            );
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            hash_str.hash(&mut hasher);
            let metrics_hash = format!("{:016x}", hasher.finish());

            // Get latest metrics hash for this repo
            let latest_hash: Option<String> = conn.query_row(
                "SELECT metrics_hash FROM metrics WHERE repo_id = ?1 ORDER BY captured_at DESC LIMIT 1;",
                [&report.repo.0],
                |row| row.get(0)
            ).optional().map_err(|e| crate::GitPurgeError::Config(format!("Failed to query latest metrics: {}", e)))?;

            if latest_hash.as_ref() != Some(&metrics_hash) {
                conn.execute(
                    "INSERT OR REPLACE INTO metrics (
                        run_id, repo_id, captured_at, total, active, stale, merged, unmerged, non_standard,
                        local_count, remote_count, protected, deleted, archived, restored, metrics_hash
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16);",
                    (
                        &report.id,
                        &report.repo.0,
                        &started_at_str,
                        metrics.total as i64,
                        metrics.active as i64,
                        metrics.stale as i64,
                        metrics.merged as i64,
                        metrics.unmerged as i64,
                        metrics.non_standard as i64,
                        metrics.local_count.map(|c| c as i64),
                        metrics.remote_count.map(|c| c as i64),
                        metrics.protected.map(|c| c as i64),
                        metrics.deleted.map(|c| c as i64),
                        metrics.archived.map(|c| c as i64),
                        metrics.restored.map(|c| c as i64),
                        &metrics_hash,
                    ),
                ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to insert metrics: {}", e)))?;
            }
        }

        // 3. Insert branch snapshots (applying PII email redaction to "[REDACTED]")
        if let Some(branches) = &report.branch_snapshots {
            for b in branches {
                let kind_str = match b.scope {
                    crate::model::BranchScope::Local => "local",
                    crate::model::BranchScope::Remote => "remote",
                };
                let is_merged = if b.merge_state == crate::model::MergeState::Merged {
                    1
                } else {
                    0
                };
                let is_stale = if b.activity == crate::model::Activity::Stale {
                    1
                } else {
                    0
                };
                let is_protected = if !matches!(b.protection, crate::model::Protection::Unprotected)
                {
                    1
                } else {
                    0
                };
                let is_standard = if matches!(
                    b.naming,
                    crate::model::NamingVerdict::Standard
                        | crate::model::NamingVerdict::Exempt { .. }
                ) {
                    1
                } else {
                    0
                };
                let violation_reason = match &b.naming {
                    crate::model::NamingVerdict::NonStandard { reason } => {
                        let reason_str = match reason {
                            crate::model::NamingViolation::NoCategoryPrefix => {
                                "No category prefix".to_string()
                            }
                            crate::model::NamingViolation::WrongPrefixFormat { prefix } => {
                                format!("Wrong prefix format: {}", prefix)
                            }
                            crate::model::NamingViolation::NonStandardPrefix { prefix } => {
                                format!("Non-standard prefix: {}", prefix)
                            }
                            crate::model::NamingViolation::UnknownPrefix { prefix } => {
                                format!("Unknown prefix: {}", prefix)
                            }
                        };
                        Some(reason_str)
                    }
                    _ => None,
                };
                let last_commit_at = b
                    .tip
                    .commit_date
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default();

                conn.execute(
                    "INSERT INTO branch_snapshots (
                        run_id, repo_id, ref_name, kind, tip_sha, commit_count, last_commit_at,
                        author_name, author_email, subject, upstream, ahead, behind,
                        is_merged, is_stale, is_protected, is_standard, violation_reason
                     ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17);",
                    rusqlite::params![
                        &report.id,
                        &report.repo.0,
                        &b.branch.0,
                        kind_str,
                        &b.tip.oid.0,
                        &last_commit_at,
                        &b.tip.author.name,
                        "[REDACTED]", // PII Redacted here!
                        &b.tip.subject,
                        match b.tracking.compared_against {
                            crate::model::RefBasis::Upstream => Some(format!("origin/{}", b.branch.0)),
                            crate::model::RefBasis::DefaultBranch => None,
                        },
                        b.tracking.ahead as i64,
                        b.tracking.behind as i64,
                        is_merged,
                        is_stale,
                        is_protected,
                        is_standard,
                        violation_reason,
                    ],
                ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to insert branch snapshot: {}", e)))?;
            }
        }

        Ok(())
    }

    fn get_history(&self, repo: &RepoId) -> Result<TrendHistory> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT captured_at, total, active, stale, merged, unmerged, non_standard, deleted, archived, protected
             FROM metrics
             WHERE repo_id = ?1
             ORDER BY captured_at ASC;"
        ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to prepare metrics query: {}", e)))?;

        let rows = stmt
            .query_map([&repo.0], |row| {
                let captured_at_str: String = row.get(0)?;
                let total: i64 = row.get(1)?;
                let active: i64 = row.get(2)?;
                let stale: i64 = row.get(3)?;
                let merged: i64 = row.get(4)?;
                let unmerged: i64 = row.get(5)?;
                let non_standard: i64 = row.get(6)?;
                let deleted: Option<i64> = row.get(7)?;
                let archived: Option<i64> = row.get(8)?;
                let protected: Option<i64> = row.get(9)?;

                let recorded_at = time::OffsetDateTime::parse(
                    &captured_at_str,
                    &time::format_description::well_known::Rfc3339,
                )
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc());

                Ok(TrendEntry {
                    recorded_at,
                    total_branches: total as usize,
                    merged_count: merged as usize,
                    unmerged_count: unmerged as usize,
                    stale_count: stale as usize,
                    active_count: active as usize,
                    deleted_count: deleted.unwrap_or(0) as usize,
                    archived_count: archived.unwrap_or(0) as usize,
                    non_standard_count: non_standard as usize,
                    protected_count: protected.unwrap_or(0) as usize,
                })
            })
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to query metrics: {}", e)))?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r.map_err(|e| crate::GitPurgeError::Config(format!("Row error: {}", e)))?);
        }

        Ok(TrendHistory {
            repo: repo.clone(),
            entries,
        })
    }

    fn get_recent(&self, repo: &RepoId, limit: usize) -> Result<Vec<TrendEntry>> {
        let history = self.get_history(repo)?;
        let mut entries = history.entries;
        entries.reverse(); // newest first
        entries.truncate(limit);
        entries.reverse(); // back to oldest first
        Ok(entries)
    }

    fn get_runs(&self, repo: &RepoId, limit: usize, offset: usize) -> Result<Vec<RunRecord>> {
        let rows = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT r.id, r.command, r.mode, r.started_at, r.finished_at, r.snapshot_id, r.actor,
                        m.deleted, m.archived
                 FROM runs r
                 LEFT JOIN metrics m ON r.id = m.run_id
                 WHERE r.repo_id = ?1
                 ORDER BY r.started_at DESC
                 LIMIT ?2 OFFSET ?3;"
            ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to prepare runs query: {}", e)))?;

            let rows = stmt
                .query_map(
                    rusqlite::params![&repo.0, limit as i64, offset as i64],
                    |row| {
                        let id: String = row.get(0)?;
                        let command: String = row.get(1)?;
                        let mode: String = row.get(2)?;
                        let started_at_str: String = row.get(3)?;
                        let finished_at_str: Option<String> = row.get(4)?;
                        let snapshot_id: Option<String> = row.get(5)?;
                        let actor: Option<String> = row.get(6)?;
                        let deleted: Option<i64> = row.get(7)?;
                        let archived: Option<i64> = row.get(8)?;

                        Ok((
                            id,
                            command,
                            mode,
                            started_at_str,
                            finished_at_str,
                            snapshot_id,
                            actor,
                            deleted,
                            archived,
                        ))
                    },
                )
                .map_err(|e| {
                    crate::GitPurgeError::Config(format!("Failed to query runs: {}", e))
                })?;

            let mut vec = Vec::new();
            for r in rows {
                vec.push(r.map_err(|e| {
                    crate::GitPurgeError::Config(format!("Row parsing error: {}", e))
                })?);
            }
            vec
        }; // conn lock is released here!

        let mut runs = Vec::new();
        for (
            id,
            command,
            mode,
            started_at_str,
            finished_at_str,
            snapshot_id,
            actor,
            deleted,
            archived,
        ) in rows
        {
            let started_at = time::OffsetDateTime::parse(
                &started_at_str,
                &time::format_description::well_known::Rfc3339,
            )
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());

            let finished_at = finished_at_str.and_then(|s| {
                time::OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339).ok()
            });

            let mut branches = Vec::new();
            if let Some(ref snap_id) = snapshot_id {
                // Safely load snapshot refs (this acquires the lock again, which is now safe)
                if let Ok(Some(snap)) = self.get_snapshot(&SnapshotId(snap_id.clone())) {
                    for ref_entry in snap.refs {
                        branches.push(ref_entry.branch.0);
                    }
                }
            }

            runs.push(RunRecord {
                id,
                command,
                mode,
                started_at,
                finished_at,
                snapshot_id,
                actor,
                deleted_count: deleted.unwrap_or(0) as usize,
                archived_count: archived.unwrap_or(0) as usize,
                branches,
            });
        }

        Ok(runs)
    }

    fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let created_at = snapshot
            .created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        let trigger_str = match snapshot.trigger {
            SnapshotTrigger::Manual => "manual",
            SnapshotTrigger::PreDelete => "pre-op",
            SnapshotTrigger::PreArchive => "pre-op",
            SnapshotTrigger::Scheduled => "scheduled",
        };
        let verified_at = if snapshot.verified {
            Some(created_at.clone())
        } else {
            None
        };
        let manifest_ref = format!("refs/gitpurge/meta/{}", snapshot.id.0);

        conn.execute(
            "INSERT OR REPLACE INTO snapshots (id, repo_id, created_at, trigger, ref_count, note, verified_at, manifest_ref)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7);",
            (
                &snapshot.id.0,
                &snapshot.repo.0,
                created_at,
                trigger_str,
                snapshot.refs.len() as i64,
                verified_at,
                manifest_ref,
            ),
        ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to save snapshot: {}", e)))?;
        Ok(())
    }

    fn list_snapshots(&self, repo: &RepoId) -> Result<Vec<Snapshot>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, created_at, trigger, ref_count, verified_at FROM snapshots WHERE repo_id = ?1 ORDER BY created_at DESC;"
        ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to prepare snapshots list: {}", e)))?;

        let rows = stmt
            .query_map([&repo.0], |row| {
                let id: String = row.get(0)?;
                let created_at_str: String = row.get(1)?;
                let trigger_str: String = row.get(2)?;
                let _ref_count: i64 = row.get(3)?;
                let verified_at: Option<String> = row.get(4)?;

                let created_at = time::OffsetDateTime::parse(
                    &created_at_str,
                    &time::format_description::well_known::Rfc3339,
                )
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
                let trigger = match trigger_str.as_str() {
                    "pre-op" => SnapshotTrigger::PreDelete,
                    "scheduled" => SnapshotTrigger::Scheduled,
                    _ => SnapshotTrigger::Manual,
                };

                Ok((SnapshotId(id), created_at, trigger, verified_at.is_some()))
            })
            .map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to query snapshots: {}", e))
            })?;

        let mut result = Vec::new();
        for r in rows {
            let (id, _created_at, _trigger, verified) =
                r.map_err(|e| crate::GitPurgeError::Config(format!("Row error: {}", e)))?;
            let mut snapshot = self.load_snapshot_from_mirror(repo, &id)?;
            snapshot.verified = verified;
            result.push(snapshot);
        }

        Ok(result)
    }

    fn get_snapshot(&self, id: &SnapshotId) -> Result<Option<Snapshot>> {
        let conn = self.conn.lock().unwrap();
        let row_opt = conn
            .query_row(
                "SELECT repo_id, created_at, trigger, verified_at FROM snapshots WHERE id = ?1;",
                [&id.0],
                |row| {
                    let repo_id_str: String = row.get(0)?;
                    let created_at_str: String = row.get(1)?;
                    let trigger_str: String = row.get(2)?;
                    let verified_at: Option<String> = row.get(3)?;

                    let repo_id = RepoId(repo_id_str);
                    let created_at = time::OffsetDateTime::parse(
                        &created_at_str,
                        &time::format_description::well_known::Rfc3339,
                    )
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
                    let trigger = match trigger_str.as_str() {
                        "pre-op" => SnapshotTrigger::PreDelete,
                        "scheduled" => SnapshotTrigger::Scheduled,
                        _ => SnapshotTrigger::Manual,
                    };
                    Ok((repo_id, created_at, trigger, verified_at.is_some()))
                },
            )
            .optional()
            .map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to get snapshot details: {}", e))
            })?;

        if let Some((repo_id, _created_at, _trigger, verified)) = row_opt {
            let mut snapshot = self.load_snapshot_from_mirror(&repo_id, id)?;
            snapshot.verified = verified;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    fn delete_snapshot(&self, id: &SnapshotId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM snapshots WHERE id = ?1;", [&id.0])
            .map_err(|e| {
                crate::GitPurgeError::Config(format!("Failed to delete snapshot row: {}", e))
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BranchName, MergeState, Oid, RepoId, Snapshot, SnapshotId, SnapshotRef, SnapshotTrigger,
    };
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_sqlite_history_store_snapshot_flow() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("history.db");
        let backups_root = temp_dir.path().join("backups");
        fs::create_dir_all(&backups_root).unwrap();

        let store = SqliteHistoryStore::new(&db_path, backups_root.clone()).unwrap();

        let repo_id = RepoId("test-repo".to_string());
        let repo = Repository {
            id: repo_id.clone(),
            display_name: "Test Repo".to_string(),
            local_path: Some(PathBuf::from("/dummy/path")),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };

        // 1. Save repo first (foreign key constraint)
        store.save_repo(&repo).unwrap();

        let snapshot_id = SnapshotId("01J8ZK9Q2F3M4N5P6R7S8T9V0W".to_string());
        let created_at = time::OffsetDateTime::now_utc();
        let manifest_path = store
            .resolve_mirror_path(&repo_id)
            .join(format!("{}.json", snapshot_id.0));

        let snapshot_ref = SnapshotRef {
            branch: BranchName("main".to_string()),
            original_full_ref: "refs/heads/main".to_string(),
            backup_ref: format!("refs/gitpurge/backups/{}/refs/heads/main", snapshot_id.0),
            tip: Oid("1234567890abcdef1234567890abcdef12345678".to_string()),
            commit_count: 10,
            upstream: None,
            merged_at_capture: MergeState::Unknown,
        };

        let snapshot = Snapshot {
            id: snapshot_id.clone(),
            repo: repo_id.clone(),
            created_at,
            trigger: SnapshotTrigger::Manual,
            refs: vec![snapshot_ref],
            verified: true, // We want to verify that this is saved correctly in SQLite, and overridden on load
            manifest_path: manifest_path.clone(),
        };

        // 2. Save snapshot in DB (inserts into snapshots table)
        store.save_snapshot(&snapshot).unwrap();

        // 3. Since bare mirror and meta ref don't exist yet, load_snapshot_from_mirror should fail.
        // Assert that list_snapshots returns Err (propagating the error).
        let list_err = store.list_snapshots(&repo_id);
        assert!(
            list_err.is_err(),
            "Expected error from missing bare mirror, got: {:?}",
            list_err
        );

        let get_err = store.get_snapshot(&snapshot_id);
        assert!(
            get_err.is_err(),
            "Expected error from missing bare mirror, got: {:?}",
            get_err
        );

        // 4. Create the bare mirror repository and meta ref
        let mirror_path = store.resolve_mirror_path(&repo_id);
        fs::create_dir_all(&mirror_path).unwrap();
        let mirror_repo = git2::Repository::init_bare(&mirror_path).unwrap();

        // Still should fail because meta ref is missing
        let list_err2 = store.list_snapshots(&repo_id);
        assert!(
            list_err2.is_err(),
            "Expected error from missing meta ref, got: {:?}",
            list_err2
        );

        // 5. Write the manifest JSON to the mirror repo as a blob and set the meta reference
        // We write the manifest with verified = false to ensure list_snapshots/get_snapshot correctly overwrite it with verified = true from DB.
        let mut manifest_snap = snapshot.clone();
        manifest_snap.verified = false;
        let json_bytes = serde_json::to_vec_pretty(&manifest_snap).unwrap();
        let blob_oid = mirror_repo.blob(&json_bytes).unwrap();
        let meta_ref_path = format!("refs/gitpurge/meta/{}", snapshot_id.0);
        mirror_repo
            .reference(&meta_ref_path, blob_oid, true, "Test meta")
            .unwrap();

        // 6. Now both list_snapshots and get_snapshot should succeed!
        let list_ok = store.list_snapshots(&repo_id).unwrap();
        assert_eq!(list_ok.len(), 1);
        assert_eq!(list_ok[0].id, snapshot_id);
        assert_eq!(list_ok[0].refs.len(), 1);
        assert_eq!(list_ok[0].refs[0].branch.0, "main");
        // Check that verified status was set from database value (which was saved as true)
        assert!(
            list_ok[0].verified,
            "Expected snapshot verified to be true (set from DB)"
        );

        let get_ok = store.get_snapshot(&snapshot_id).unwrap();
        assert!(get_ok.is_some());
        let loaded = get_ok.unwrap();
        assert_eq!(loaded.id, snapshot_id);
        assert!(loaded.verified);

        // 7. Delete the snapshot and verify
        store.delete_snapshot(&snapshot_id).unwrap();
        let get_none = store.get_snapshot(&snapshot_id).unwrap();
        assert!(get_none.is_none());
    }

    #[test]
    fn test_sqlite_history_store_runs_and_trends_flow() {
        use crate::model::{
            Activity, BranchScope, Classification, Commit, ExecMode, NamingVerdict, Oid,
            Protection, Recommendation, RefBasis, RepoId, RunMetrics, RunReport, Signature,
            TrackingFacet,
        };

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("history.db");
        let backups_root = temp_dir.path().join("backups");
        fs::create_dir_all(&backups_root).unwrap();

        let store = SqliteHistoryStore::new(&db_path, backups_root).unwrap();

        let repo_id = RepoId("test-repo-trends".to_string());
        let repo = Repository {
            id: repo_id.clone(),
            display_name: "Test Repo Trends".to_string(),
            local_path: Some(PathBuf::from("/dummy/path")),
            remote_url: None,
            default_branch: None,
            provider: crate::model::ProviderHint::Unknown,
            added_at: time::OffsetDateTime::now_utc(),
            last_scanned_at: None,
        };

        store.save_repo(&repo).unwrap();

        let author = Signature {
            name: "John Doe".to_string(),
            email: "john.doe@example.com".to_string(),
            when: time::OffsetDateTime::now_utc(),
        };

        let commit = Commit {
            oid: Oid("1111111111222222222233333333334444444444".to_string()),
            short: "1111111".to_string(),
            author: author.clone(),
            committer: author.clone(),
            author_date: time::OffsetDateTime::now_utc(),
            commit_date: time::OffsetDateTime::now_utc(),
            subject: "Initial commit".to_string(),
            parents: Vec::new(),
        };

        let branch_class = Classification {
            branch: BranchName("feat/login".to_string()),
            scope: BranchScope::Local,
            remote: None,
            upstream: None,
            merge_state: MergeState::Merged,
            activity: Activity::Active,
            age: std::time::Duration::from_secs(3600),
            protection: Protection::Unprotected,
            naming: NamingVerdict::Standard,
            tracking: TrackingFacet {
                ahead: 0,
                behind: 0,
                upstream_gone: false,
                compared_against: RefBasis::DefaultBranch,
            },
            tip: commit.clone(),
            recommendation: Recommendation::NoAction,
        };

        let metrics = RunMetrics {
            total: 10,
            active: 5,
            stale: 5,
            merged: 4,
            unmerged: 6,
            non_standard: 2,
            local_count: Some(6),
            remote_count: Some(4),
            protected: Some(1),
            deleted: Some(0),
            archived: Some(0),
            restored: Some(0),
        };

        let report1 = RunReport {
            id: "run-01".to_string(),
            repo: repo_id.clone(),
            command: "scan".to_string(),
            mode: ExecMode::DryRun,
            started_at: time::OffsetDateTime::now_utc(),
            snapshot: None,
            metrics: Some(metrics.clone()),
            branch_snapshots: Some(vec![branch_class.clone()]),
            results: Vec::new(),
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
        };

        // 1. Record first run
        store.record_run(&report1).unwrap();

        // Verify that PII (author email) was redacted
        {
            let conn = store.conn.lock().unwrap();
            let email: String = conn
                .query_row(
                    "SELECT author_email FROM branch_snapshots WHERE run_id = 'run-01' LIMIT 1;",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(email, "[REDACTED]", "Author email must be redacted");
        }

        // Verify get_history and get_recent return the recorded entry
        let history = store.get_history(&repo_id).unwrap();
        assert_eq!(history.entries.len(), 1);
        assert_eq!(history.entries[0].total_branches, 10);
        assert_eq!(history.entries[0].stale_count, 5);

        let recent = store.get_recent(&repo_id, 5).unwrap();
        assert_eq!(recent.len(), 1);

        // 2. Record second run with identical metrics (should be deduplicated in metrics table)
        let report2 = RunReport {
            id: "run-02".to_string(),
            repo: repo_id.clone(),
            command: "scan".to_string(),
            mode: ExecMode::DryRun,
            started_at: time::OffsetDateTime::now_utc() + time::Duration::seconds(10),
            snapshot: None,
            metrics: Some(metrics.clone()),
            branch_snapshots: Some(vec![branch_class.clone()]),
            results: Vec::new(),
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
        };
        store.record_run(&report2).unwrap();

        // History count in metrics table should still be 1 due to deduplication
        let history_dup = store.get_history(&repo_id).unwrap();
        assert_eq!(
            history_dup.entries.len(),
            1,
            "Duplicate metrics must be deduplicated"
        );

        // Create mock snapshot and write it to bare mirror
        let snapshot_id = SnapshotId("01J8ZK9Q2F3M4N5P6R7S8T9V0W".to_string());
        let created_at = time::OffsetDateTime::now_utc();
        let manifest_path = store
            .resolve_mirror_path(&repo_id)
            .join(format!("{}.json", snapshot_id.0));

        let snapshot_ref = SnapshotRef {
            branch: BranchName("feat/login".to_string()),
            original_full_ref: "refs/heads/feat/login".to_string(),
            backup_ref: format!(
                "refs/gitpurge/backups/{}/refs/heads/feat/login",
                snapshot_id.0
            ),
            tip: commit.oid.clone(),
            commit_count: 10,
            upstream: None,
            merged_at_capture: MergeState::Unknown,
        };

        let snapshot = Snapshot {
            id: snapshot_id.clone(),
            repo: repo_id.clone(),
            created_at,
            trigger: SnapshotTrigger::Manual,
            refs: vec![snapshot_ref],
            verified: true,
            manifest_path: manifest_path.clone(),
        };

        // Create the bare mirror repository and meta ref
        let mirror_path = store.resolve_mirror_path(&repo_id);
        fs::create_dir_all(&mirror_path).unwrap();
        let mirror_repo = git2::Repository::init_bare(&mirror_path).unwrap();

        let json_bytes = serde_json::to_vec_pretty(&snapshot).unwrap();
        let blob_oid = mirror_repo.blob(&json_bytes).unwrap();
        let meta_ref_path = format!("refs/gitpurge/meta/{}", snapshot_id.0);
        mirror_repo
            .reference(&meta_ref_path, blob_oid, true, "Test meta")
            .unwrap();

        // Save snapshot metadata in DB
        store.save_snapshot(&snapshot).unwrap();

        // 3. Record third run with modified metrics (should insert a new metrics row)
        let mut modified_metrics = metrics.clone();
        modified_metrics.stale = 6;
        let report3 = RunReport {
            id: "run-03".to_string(),
            repo: repo_id.clone(),
            command: "scan".to_string(),
            mode: ExecMode::DryRun,
            started_at: time::OffsetDateTime::now_utc() + time::Duration::seconds(20),
            snapshot: Some(snapshot_id),
            metrics: Some(modified_metrics),
            branch_snapshots: Some(vec![branch_class]),
            results: Vec::new(),
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
        };
        store.record_run(&report3).unwrap();

        // History count in metrics table should now be 2
        let history_mod = store.get_history(&repo_id).unwrap();
        assert_eq!(
            history_mod.entries.len(),
            2,
            "Modified metrics must be inserted"
        );
        assert_eq!(history_mod.entries[1].stale_count, 6);

        // Verify get_runs works with pagination and ordering DESC
        let runs = store.get_runs(&repo_id, 10, 0).unwrap();
        assert_eq!(runs.len(), 3, "Should fetch all 3 runs");
        assert_eq!(runs[0].id, "run-03");
        assert_eq!(runs[0].branches.len(), 1);
        assert_eq!(runs[0].branches[0], "feat/login");
        assert_eq!(runs[1].id, "run-02");
        assert_eq!(runs[2].id, "run-01");

        // Verify limit and offset pagination
        let paginated_runs = store.get_runs(&repo_id, 1, 1).unwrap();
        assert_eq!(paginated_runs.len(), 1);
        assert_eq!(
            paginated_runs[0].id, "run-02",
            "Should respect limit and offset"
        );
    }
}
