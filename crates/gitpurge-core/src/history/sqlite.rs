//! SQLite history store implementation (CONVENTIONS §5, doc 10 §3)

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use rusqlite::{Connection, OptionalExtension};

use crate::error::Result;
use crate::model::{
    RepoId, Repository, RunReport, Snapshot, SnapshotId, SnapshotTrigger,
    TrendEntry, TrendHistory,
};
use crate::history::HistoryStore;

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
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to enable foreign keys: {}", e)))?;
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

    fn load_snapshot_from_mirror(&self, repo_id: &RepoId, snapshot_id: &SnapshotId) -> Result<Snapshot> {
        let mirror_path = self.resolve_mirror_path(repo_id);
        if !mirror_path.exists() {
            return Err(crate::GitPurgeError::Snapshot(format!("Bare mirror not found: {:?}", mirror_path)));
        }
        let mirror_repo = git2::Repository::open_bare(&mirror_path)
            .map_err(|e| crate::GitPurgeError::Git(format!("Failed to open bare mirror: {}", e)))?;
        let meta_ref_path = format!("refs/gitpurge/meta/{}", snapshot_id.0);
        let meta_ref = mirror_repo.find_reference(&meta_ref_path)
            .map_err(|e| crate::GitPurgeError::Snapshot(format!("Snapshot meta ref not found: {}", e)))?;
        let oid = meta_ref.target().ok_or_else(|| {
            crate::GitPurgeError::Snapshot("Meta reference points to no OID".to_string())
        })?;
        let blob = mirror_repo.find_blob(oid)
            .map_err(|e| crate::GitPurgeError::Snapshot(format!("Failed to find metadata blob: {}", e)))?;
        let snapshot: Snapshot = serde_json::from_slice(blob.content())
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to parse manifest: {}", e)))?;
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

        let default_branch = repo.default_branch.as_ref().map(|b| b.name.0.clone()).unwrap_or_else(|| "origin/main".to_string());
        let created_at = repo.added_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default();

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
        let finished_at = time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or_default();
        let started_at_str = report.started_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default();
        let actor = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "system".to_string());
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
            let hash_str = format!("{}:{}:{}:{}:{}:{}", metrics.total, metrics.active, metrics.stale, metrics.merged, metrics.unmerged, metrics.non_standard);
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
                let is_merged = if b.merge_state == crate::model::MergeState::Merged { 1 } else { 0 };
                let is_stale = if b.activity == crate::model::Activity::Stale { 1 } else { 0 };
                let is_protected = if !matches!(b.protection, crate::model::Protection::Unprotected) { 1 } else { 0 };
                let is_standard = if matches!(b.naming, crate::model::NamingVerdict::Standard | crate::model::NamingVerdict::Exempt { .. }) { 1 } else { 0 };
                let violation_reason = match &b.naming {
                    crate::model::NamingVerdict::NonStandard { reason } => {
                        let reason_str = match reason {
                            crate::model::NamingViolation::NoCategoryPrefix => "No category prefix".to_string(),
                            crate::model::NamingViolation::WrongPrefixFormat { prefix } => format!("Wrong prefix format: {}", prefix),
                            crate::model::NamingViolation::NonStandardPrefix { prefix } => format!("Non-standard prefix: {}", prefix),
                            crate::model::NamingViolation::UnknownPrefix { prefix } => format!("Unknown prefix: {}", prefix),
                        };
                        Some(reason_str)
                    }
                    _ => None,
                };
                let last_commit_at = b.tip.commit_date.format(&time::format_description::well_known::Rfc3339).unwrap_or_default();

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
            "SELECT captured_at, total, active, stale, merged, unmerged, non_standard, deleted, archived
             FROM metrics
             WHERE repo_id = ?1
             ORDER BY captured_at ASC;"
        ).map_err(|e| crate::GitPurgeError::Config(format!("Failed to prepare metrics query: {}", e)))?;

        let rows = stmt.query_map([&repo.0], |row| {
            let captured_at_str: String = row.get(0)?;
            let total: i64 = row.get(1)?;
            let active: i64 = row.get(2)?;
            let stale: i64 = row.get(3)?;
            let merged: i64 = row.get(4)?;
            let unmerged: i64 = row.get(5)?;
            let non_standard: i64 = row.get(6)?;
            let deleted: Option<i64> = row.get(7)?;
            let archived: Option<i64> = row.get(8)?;

            let recorded_at = time::OffsetDateTime::parse(&captured_at_str, &time::format_description::well_known::Rfc3339)
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
            })
        }).map_err(|e| crate::GitPurgeError::Config(format!("Failed to query metrics: {}", e)))?;

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

    fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let created_at = snapshot.created_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default();
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

        let rows = stmt.query_map([&repo.0], |row| {
            let id: String = row.get(0)?;
            let created_at_str: String = row.get(1)?;
            let trigger_str: String = row.get(2)?;
            let _ref_count: i64 = row.get(3)?;
            let verified_at: Option<String> = row.get(4)?;

            let created_at = time::OffsetDateTime::parse(&created_at_str, &time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
            let trigger = match trigger_str.as_str() {
                "pre-op" => SnapshotTrigger::PreDelete,
                "scheduled" => SnapshotTrigger::Scheduled,
                _ => SnapshotTrigger::Manual,
            };

            Ok((SnapshotId(id), created_at, trigger, verified_at.is_some()))
        }).map_err(|e| crate::GitPurgeError::Config(format!("Failed to query snapshots: {}", e)))?;

        let mut result = Vec::new();
        for r in rows {
            let (id, created_at, trigger, verified) = r.map_err(|e| crate::GitPurgeError::Config(format!("Row error: {}", e)))?;
            let snapshot = self.load_snapshot_from_mirror(repo, &id).unwrap_or_else(|_| {
                Snapshot {
                    id: id.clone(),
                    repo: repo.clone(),
                    created_at,
                    trigger,
                    refs: Vec::new(),
                    verified,
                    manifest_path: self.resolve_mirror_path(repo).join("snapshot.json"),
                }
            });
            result.push(snapshot);
        }

        Ok(result)
    }

    fn get_snapshot(&self, id: &SnapshotId) -> Result<Option<Snapshot>> {
        let conn = self.conn.lock().unwrap();
        let row_opt = conn.query_row(
            "SELECT repo_id, created_at, trigger, verified_at FROM snapshots WHERE id = ?1;",
            [&id.0],
            |row| {
                let repo_id_str: String = row.get(0)?;
                let created_at_str: String = row.get(1)?;
                let trigger_str: String = row.get(2)?;
                let verified_at: Option<String> = row.get(3)?;

                let repo_id = RepoId(repo_id_str);
                let created_at = time::OffsetDateTime::parse(&created_at_str, &time::format_description::well_known::Rfc3339)
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
                let trigger = match trigger_str.as_str() {
                    "pre-op" => SnapshotTrigger::PreDelete,
                    "scheduled" => SnapshotTrigger::Scheduled,
                    _ => SnapshotTrigger::Manual,
                };
                Ok((repo_id, created_at, trigger, verified_at.is_some()))
            }
        ).optional().map_err(|e| crate::GitPurgeError::Config(format!("Failed to get snapshot details: {}", e)))?;

        if let Some((repo_id, created_at, trigger, verified)) = row_opt {
            let snapshot = self.load_snapshot_from_mirror(&repo_id, id).unwrap_or_else(|_| {
                Snapshot {
                    id: id.clone(),
                    repo: repo_id.clone(),
                    created_at,
                    trigger,
                    refs: Vec::new(),
                    verified,
                    manifest_path: self.resolve_mirror_path(&repo_id).join("snapshot.json"),
                }
            });
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    fn delete_snapshot(&self, id: &SnapshotId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM snapshots WHERE id = ?1;", [&id.0])
            .map_err(|e| crate::GitPurgeError::Config(format!("Failed to delete snapshot row: {}", e)))?;
        Ok(())
    }
}
