use tauri::{AppHandle, Emitter};

use gitpurge_core::{
    model::{ActionResult, Plan, Repository, RunReport, ScanOptions, ScanResult, Snapshot},
    Config, Engine, GitPurgeError,
};

// --- TS/IPC Projections ---

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SerializableError {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoSummary {
    pub id: String,
    pub name: String,
    pub local_path: Option<String>,
    pub remote_url: Option<String>,
    pub branch_count: usize,
    pub last_scanned: Option<String>,
    pub stale: usize,
    pub unmerged: usize,
    pub protected_count: usize,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoDetail {
    #[serde(flatten)]
    pub summary: RepoSummary,
    pub default_branch: String,
    pub remotes: Vec<String>,
    pub backup_count: usize,
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientScanOptions {
    pub age: Option<String>,
    pub naming: Option<bool>,
    pub include_remote: Option<bool>,
    pub auto_fetch: Option<bool>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    pub name: String,
    pub ref_path: String,
    pub tip_sha: String,
    pub tip_short: String,
    pub author_name: String,
    pub committed_at: String,
    pub age_days: u64,
    pub upstream: Option<String>,
    pub classification: Classification,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Classification {
    pub merge: String,     // 'merged' | 'unmerged'
    pub locality: String,  // 'local' | 'remote'
    pub freshness: String, // 'stale' | 'active'
    pub protected: bool,
    pub naming: String, // 'standard' | 'nonStandard'
    pub ahead: u32,
    pub behind: u32,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientScanResult {
    pub repo_id: String,
    pub scanned_at: String,
    pub branches: Vec<Branch>,
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientActionFilter {
    pub kind: String, // 'delete' | 'archive'
    pub age: Option<String>,
    pub merged: Option<bool>,
    pub include_unmerged: Option<bool>,
    pub exclude: Option<Vec<String>>,
    pub refs: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientPlannedAction {
    pub ref_name: String,
    pub action: String,
    pub reason: String,
    pub classification: Classification,
    pub destructive: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientPlan {
    pub repo_id: String,
    pub kind: String,
    pub actions: Vec<ClientPlannedAction>,
    pub created_at: String,
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientExecOptions {
    pub no_backup: bool,
    pub confirmed_token: Option<String>,
    pub target_branch: Option<String>,
    pub strategy: Option<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientRunReport {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub snapshot_id: Option<String>,
    pub per_ref: Vec<RefOutcome>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RefOutcome {
    pub ref_name: String,
    pub outcome: String, // 'done' | 'failed' | 'skipped' | 'restored'
    pub error: Option<SerializableError>,
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientBackupOptions {
    pub trigger: String, // 'manual' | 'preDelete' | 'scheduled'
    pub refs: Option<Vec<String>>,
    pub verify: bool,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientSnapshot {
    pub id: String,
    pub repo_id: String,
    pub created_at: String,
    pub trigger: String, // 'manual' | 'preDelete' | 'scheduled'
    pub ref_count: usize,
    pub verified: bool,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientSnapshotDetail {
    #[serde(flatten)]
    pub snapshot: ClientSnapshot,
    pub refs: Vec<ClientSnapshotRef>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientSnapshotRef {
    pub branch: String,
    pub tip_sha: String,
    pub commit_count: usize,
    pub upstream: Option<String>,
    pub merge: String,
    pub locality: String,
    pub original_ref: Option<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientVerifyReport {
    pub snapshot_id: String,
    pub ok: bool,
    pub checked_refs: usize,
    pub problems: Vec<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientPruneReport {
    pub removed: Vec<String>,
    pub kept: Vec<String>,
    pub reclaimed_bytes: u64,
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientRestoreSpec {
    pub ref_name: String,
    pub target_type: String, // 'branch' | 'tag'
    pub new_name: Option<String>,
    pub force: bool,
    pub original_ref: Option<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientRestoreOutcome {
    pub restored: String,
    pub r#as: String,
    pub sha: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientDiffResult {
    pub a: ClientRefSpec,
    pub b: ClientRefSpec,
    pub files: Vec<DiffFile>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientRefSpec {
    pub repo_id: String,
    pub r#ref: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiffFile {
    pub path: String,
    pub status: String,
    pub added: usize,
    pub removed: usize,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientTreeView {
    pub at: ClientRefSpec,
    pub path: String,
    pub entries: Vec<ClientTreeEntry>,
    pub blob: Option<BlobData>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientTreeEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub size: Option<u64>,
    pub mode: String,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlobData {
    pub text: String,
    pub truncated: bool,
    pub binary: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub policy: PolicySettings,
    pub backups_root: String,
    pub default_no_backup: bool,
    pub date_format: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicySettings {
    pub age: String,
    pub naming_regex: String,
    pub protected_refs: Vec<String>,
    pub exclude_globs: Vec<String>,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientProgressEvent {
    pub task_id: String,
    pub phase: String,
    pub message: String,
    pub current: u64,
    pub total: u64,
    pub done: bool,
    pub error: Option<SerializableError>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActiveCleanupTask {
    pub task_id: String,
    pub repo_id: String,
    pub kind: String, // "delete" | "archive"
    pub status: String, // "running" | "completed" | "failed" | "cancelled"
    pub current: u64,
    pub total: u64,
    pub message: String,
    pub started_at: String,
}

// --- Helper Mappings ---

pub fn map_error(err: GitPurgeError) -> SerializableError {
    let (code, msg, hint) = match err {
        GitPurgeError::Config(m) => (
            "CONFIG",
            m,
            Some("Check your config.toml layout or CLI override path.".to_string()),
        ),
        GitPurgeError::RepoNotFound(m) | GitPurgeError::RefNotFound(m) => (
            "NOT_FOUND",
            m,
            Some("Verify that the repository is registered and the reference exists.".to_string()),
        ),
        GitPurgeError::Git(m) | GitPurgeError::BackendUnsupported(m) => (
            "GIT",
            m,
            Some(
                "Ensure the repository is not corrupted and remote access is available."
                    .to_string(),
            ),
        ),
        GitPurgeError::Auth(m) => (
            "AUTH",
            m,
            Some("Check your credentials or SSH keys/agents.".to_string()),
        ),
        GitPurgeError::SafetyViolation(m) => (
            "SAFETY",
            m,
            Some("Branch is protected, has unmerged commits, or is current HEAD.".to_string()),
        ),
        GitPurgeError::Snapshot(m) => (
            "BACKUP",
            m,
            Some("A backup snapshot failed integrity checks. Mutations aborted.".to_string()),
        ),
        GitPurgeError::History(m) => (
            "PARTIAL",
            m,
            Some("History store error occurred.".to_string()),
        ),
        GitPurgeError::Io(e) => (
            "IO",
            e.to_string(),
            Some("Check file and directory permissions.".to_string()),
        ),
        GitPurgeError::Other(m) => (
            "GENERIC",
            m,
            Some("An unexpected internal error occurred.".to_string()),
        ),
        _ => (
            "GENERIC",
            "An unexpected error occurred.".to_string(),
            Some("Verify logs and configuration.".to_string()),
        ),
    };

    SerializableError {
        code: code.to_string(),
        message: msg,
        hint,
    }
}

pub fn map_repo_summary(engine: &Engine, repo: &Repository) -> RepoSummary {
    let counts = if let Ok(history) = engine.history(&repo.id) {
        if let Some(latest) = history.entries.last() {
            (
                latest.total_branches,
                latest.stale_count,
                latest.unmerged_count,
                latest.protected_count,
            )
        } else {
            if let Ok(scan_res) = engine.scan(&repo.id, ScanOptions::default()) {
                let stale = scan_res
                    .classifications
                    .iter()
                    .filter(|c| matches!(c.activity, gitpurge_core::model::Activity::Stale))
                    .count();
                let unmerged = scan_res
                    .classifications
                    .iter()
                    .filter(|c| matches!(c.merge_state, gitpurge_core::model::MergeState::Unmerged))
                    .count();
                let protected = scan_res
                    .classifications
                    .iter()
                    .filter(|c| {
                        !matches!(c.protection, gitpurge_core::model::Protection::Unprotected)
                    })
                    .count();
                (scan_res.total_branches, stale, unmerged, protected)
            } else {
                (0, 0, 0, 0)
            }
        }
    } else {
        (0, 0, 0, 0)
    };

    RepoSummary {
        id: repo.id.0.clone(),
        name: repo.display_name.clone(),
        local_path: repo
            .local_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        remote_url: repo.remote_url.as_ref().map(|u| u.raw.clone()),
        branch_count: counts.0,
        last_scanned: repo.last_scanned_at.map(format_datetime),
        stale: counts.1,
        unmerged: counts.2,
        protected_count: counts.3,
    }
}

pub fn map_repo_detail(engine: &Engine, repo: &Repository) -> RepoDetail {
    let summary = map_repo_summary(engine, repo);
    let default_branch = repo
        .default_branch
        .as_ref()
        .map(|b| b.name.0.clone())
        .unwrap_or_else(|| "main".to_string());
    let backup_count = engine
        .list_snapshots(&repo.id)
        .map(|l| l.len())
        .unwrap_or(0);

    RepoDetail {
        summary,
        default_branch,
        remotes: repo
            .remote_url
            .as_ref()
            .map(|_| vec!["origin".to_string()])
            .unwrap_or_default(),
        backup_count,
    }
}

pub fn map_classification(c: &gitpurge_core::model::Classification) -> Classification {
    let merge = match c.merge_state {
        gitpurge_core::model::MergeState::Merged => "merged",
        _ => "unmerged",
    }
    .to_string();

    let locality = match c.scope {
        gitpurge_core::model::BranchScope::Local => "local",
        gitpurge_core::model::BranchScope::Remote => "remote",
    }
    .to_string();

    let freshness = match c.activity {
        gitpurge_core::model::Activity::Stale => "stale",
        gitpurge_core::model::Activity::Active => "active",
    }
    .to_string();

    let naming = match c.naming {
        gitpurge_core::model::NamingVerdict::Standard
        | gitpurge_core::model::NamingVerdict::Exempt { .. } => "standard",
        _ => "nonStandard",
    }
    .to_string();

    let protected = !matches!(c.protection, gitpurge_core::model::Protection::Unprotected);

    Classification {
        merge,
        locality,
        freshness,
        protected,
        naming,
        ahead: c.tracking.ahead,
        behind: c.tracking.behind,
    }
}

pub fn map_scan_result(scan_res: ScanResult) -> ClientScanResult {
    let branches = scan_res
        .classifications
        .into_iter()
        .map(|c| {
            let is_remote = c.scope == gitpurge_core::model::BranchScope::Remote;
            let remote_name = c.remote.as_deref().unwrap_or("origin");
            let display_name = if is_remote {
                format!("{}/{}", remote_name, c.branch.0)
            } else {
                c.branch.0.clone()
            };
            let ref_path = if is_remote {
                format!("refs/remotes/{}/{}", remote_name, c.branch.0)
            } else {
                format!("refs/heads/{}", c.branch.0)
            };
            Branch {
                name: display_name,
                ref_path,
                tip_sha: c.tip.oid.0.clone(),
                tip_short: c.tip.short.clone(),
                author_name: c.tip.author.name.clone(),
                committed_at: format_datetime(c.tip.commit_date),
                age_days: c.age.as_secs() / 86400,
                upstream: if is_remote {
                    None
                } else {
                    c.upstream
                        .as_ref()
                        .map(|u| format!("{}/{}", u.remote, u.ref_name.0))
                },
                classification: map_classification(&c),
            }
        })
        .collect();

    ClientScanResult {
        repo_id: scan_res.repo.0,
        scanned_at: format_datetime(time::OffsetDateTime::now_utc()),
        branches,
    }
}

pub fn map_plan(core_plan: Plan) -> ClientPlan {
    let actions = core_plan
        .actions
        .into_iter()
        .map(|a| {
            let ref_name = if a.scope == gitpurge_core::model::BranchScope::Remote {
                let remote_name = a.remote.as_deref().unwrap_or("origin");
                format!("{}/{}", remote_name, a.branch.0)
            } else {
                a.branch.0.clone()
            };
            ClientPlannedAction {
                ref_name,
                action: match a.kind {
                    gitpurge_core::model::ActionKind::Delete => "delete",
                    gitpurge_core::model::ActionKind::Archive => "archive",
                    _ => "delete",
                }
                .to_string(),
                reason: a.reason,
                destructive: a.classification.merge_state
                    == gitpurge_core::model::MergeState::Unmerged,
                classification: map_classification(&a.classification),
            }
        })
        .collect();

    ClientPlan {
        repo_id: core_plan.repo.0,
        kind: "delete".to_string(),
        actions,
        created_at: format_datetime(time::OffsetDateTime::now_utc()),
    }
}

pub fn map_run_report(core_report: RunReport) -> ClientRunReport {
    let per_ref = core_report
        .results
        .into_iter()
        .map(|r| {
            let (ref_name, outcome, error) = match r {
                ActionResult::Success { action, .. } => {
                    let name = if action.scope == gitpurge_core::model::BranchScope::Remote {
                        format!("origin/{}", action.branch.0)
                    } else {
                        action.branch.0.clone()
                    };
                    (name, "done".to_string(), None)
                }
                ActionResult::Failed { action, error } => {
                    let name = if action.scope == gitpurge_core::model::BranchScope::Remote {
                        format!("origin/{}", action.branch.0)
                    } else {
                        action.branch.0.clone()
                    };
                    (
                        name,
                        "failed".to_string(),
                        Some(SerializableError {
                            code: "GIT_ERROR".to_string(),
                            message: error.clone(),
                            hint: None,
                        }),
                    )
                }
                ActionResult::Skipped { action } => {
                    let name = if action.scope == gitpurge_core::model::BranchScope::Remote {
                        format!("origin/{}", action.branch.0)
                    } else {
                        action.branch.0.clone()
                    };
                    (name, "skipped".to_string(), None)
                }
            };
            RefOutcome {
                ref_name,
                outcome,
                error,
            }
        })
        .collect();

    ClientRunReport {
        run_id: ulid::Ulid::new().to_string(),
        started_at: format_datetime(time::OffsetDateTime::now_utc()),
        finished_at: format_datetime(time::OffsetDateTime::now_utc()),
        attempted: core_report.success_count + core_report.failure_count,
        succeeded: core_report.success_count,
        failed: core_report.failure_count,
        skipped: core_report.skipped_count,
        snapshot_id: core_report.snapshot.map(|s| s.0),
        per_ref,
    }
}

pub fn map_snapshot(core_snap: &Snapshot) -> ClientSnapshot {
    ClientSnapshot {
        id: core_snap.id.0.clone(),
        repo_id: core_snap.repo.0.clone(),
        created_at: format_datetime(core_snap.created_at),
        trigger: match core_snap.trigger {
            gitpurge_core::model::SnapshotTrigger::Manual => "manual",
            gitpurge_core::model::SnapshotTrigger::PreDelete => "preDelete",
            gitpurge_core::model::SnapshotTrigger::PreArchive => "preDelete",
            gitpurge_core::model::SnapshotTrigger::Scheduled => "scheduled",
        }
        .to_string(),
        ref_count: core_snap.refs.len(),
        verified: core_snap.verified,
    }
}

pub fn map_snapshot_detail(core_snap: &Snapshot) -> ClientSnapshotDetail {
    let snapshot = map_snapshot(core_snap);
    let refs = core_snap
        .refs
        .iter()
        .map(|r| {
            let locality = if r.original_full_ref.starts_with("refs/remotes/") {
                "remote"
            } else {
                "local"
            };
            ClientSnapshotRef {
                branch: r.branch.0.clone(),
                tip_sha: r.tip.0.clone(),
                commit_count: r.commit_count as usize,
                upstream: r.upstream.clone(),
                merge: match r.merged_at_capture {
                    gitpurge_core::model::MergeState::Merged => "merged",
                    _ => "unmerged",
                }
                .to_string(),
                locality: locality.to_string(),
                original_ref: Some(r.original_full_ref.clone()),
            }
        })
        .collect();

    ClientSnapshotDetail { snapshot, refs }
}

pub fn map_verify_report(core_report: gitpurge_core::backup::VerifyReport) -> ClientVerifyReport {
    ClientVerifyReport {
        snapshot_id: String::new(),
        ok: core_report.ok,
        checked_refs: core_report.per_ref.len(),
        problems: core_report
            .problems
            .iter()
            .map(|p| format!("{:?}", p))
            .collect(),
    }
}

pub fn map_prune_report(core_report: gitpurge_core::model::PruneReport) -> ClientPruneReport {
    ClientPruneReport {
        removed: core_report
            .pruned_snapshots
            .into_iter()
            .map(|s| s.0)
            .collect(),
        kept: Vec::new(),
        reclaimed_bytes: core_report.space_reclaimed_bytes,
    }
}

pub fn map_restore_outcome(outcome: gitpurge_core::model::RestoreOutcome) -> ClientRestoreOutcome {
    ClientRestoreOutcome {
        restored: outcome.branch.0,
        r#as: if outcome.as_tag {
            "tag".to_string()
        } else {
            "branch".to_string()
        },
        sha: outcome.tip.0,
    }
}

pub fn unescape_git_path(path: &str) -> String {
    let mut bytes = Vec::new();
    let mut s = path;
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s = &s[1..s.len() - 1];
    }

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next.is_digit(8)
                && i + 3 < chars.len()
                && chars[i + 2].is_digit(8)
                && chars[i + 3].is_digit(8)
            {
                let octal_str: String = chars[i + 1..=i + 3].iter().collect();
                if let Ok(byte_val) = u8::from_str_radix(&octal_str, 8) {
                    bytes.push(byte_val);
                    i += 4;
                    continue;
                }
            }

            match next {
                'n' => bytes.push(b'\n'),
                'r' => bytes.push(b'\n'),
                't' => bytes.push(b'\t'),
                '\\' => bytes.push(b'\\'),
                '"' => bytes.push(b'"'),
                _ => bytes.extend_from_slice(chars[i].to_string().as_bytes()),
            }
            i += 2;
        } else {
            bytes.extend_from_slice(chars[i].to_string().as_bytes());
            i += 1;
        }
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn map_diff_result(
    repo_id: &str,
    core_res: gitpurge_core::diff::DiffResult,
) -> ClientDiffResult {
    let mut files: Vec<DiffFile> = core_res
        .entries
        .into_iter()
        .map(|e| DiffFile {
            path: unescape_git_path(&e.path),
            status: match e.kind {
                gitpurge_core::diff::DiffKind::Added => "added",
                gitpurge_core::diff::DiffKind::Deleted => "deleted",
                gitpurge_core::diff::DiffKind::Modified => "modified",
                gitpurge_core::diff::DiffKind::Renamed => "renamed",
                gitpurge_core::diff::DiffKind::Copied => "modified",
            }
            .to_string(),
            added: e.additions as usize,
            removed: e.deletions as usize,
        })
        .collect();

    files.sort_by_key(|a| a.path.to_lowercase());

    ClientDiffResult {
        a: ClientRefSpec {
            repo_id: repo_id.to_string(),
            r#ref: format!("{:?}", core_res.from),
        },
        b: ClientRefSpec {
            repo_id: repo_id.to_string(),
            r#ref: format!("{:?}", core_res.to),
        },
        files,
        ahead: core_res.insertions as usize,
        behind: core_res.deletions as usize,
    }
}

pub fn map_tree_view(
    repo_id: &str,
    core_tree: gitpurge_core::diff::TreeView,
    blob_text: Option<String>,
) -> ClientTreeView {
    let entries = core_tree
        .entries
        .into_iter()
        .map(|e| {
            let name = std::path::Path::new(&e.path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| e.path.clone());
            ClientTreeEntry {
                name,
                path: e.path.clone(),
                kind: if e.is_dir {
                    "dir".to_string()
                } else {
                    "file".to_string()
                },
                size: Some(e.size),
                mode: if e.is_dir {
                    "040000".to_string()
                } else {
                    "100644".to_string()
                },
            }
        })
        .collect();

    let blob = blob_text.map(|t| BlobData {
        text: t,
        truncated: false,
        binary: false,
    });

    ClientTreeView {
        at: ClientRefSpec {
            repo_id: repo_id.to_string(),
            r#ref: format!("{:?}", core_tree.at),
        },
        path: "".to_string(),
        entries,
        blob,
    }
}

pub fn map_settings(config: &Config) -> Settings {
    let policy = &config.default_policy;
    let naming_regex = policy
        .naming
        .allowed
        .first()
        .map(|r| r.0.clone())
        .unwrap_or_default();
    let protected_refs = policy
        .protection
        .protected_globs
        .iter()
        .map(|g| g.0.clone())
        .collect();
    let exclude_globs = policy.excludes.iter().map(|g| g.0.clone()).collect();

    let date_format = if config.date_format.trim().is_empty() {
        "YYYY-MM-DD h:m a".to_string()
    } else {
        config.date_format.clone()
    };

    Settings {
        theme: "system".to_string(),
        policy: PolicySettings {
            age: policy.age.clone(),
            naming_regex,
            protected_refs,
            exclude_globs,
        },
        backups_root: config
            .backups_root
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        default_no_backup: false,
        date_format,
    }
}

pub fn format_datetime(dt: time::OffsetDateTime) -> String {
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| dt.to_string())
}

// --- Progress Event Emitter helper ---

pub struct TauriProgressSink {
    app: AppHandle,
    task_id: String,
    phase: String,
    current: std::sync::atomic::AtomicU64,
    total: std::sync::atomic::AtomicU64,
}

impl TauriProgressSink {
    pub fn new(app: AppHandle, task_id: String, phase: String) -> Self {
        Self {
            app,
            task_id,
            phase,
            current: std::sync::atomic::AtomicU64::new(0),
            total: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl gitpurge_core::progress::ProgressSink for TauriProgressSink {
    fn set_total(&self, total: u64) {
        self.total.store(total, std::sync::atomic::Ordering::SeqCst);
    }

    fn tick(&self, message: Option<&str>) {
        let current = self
            .current
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;
        let total = self.total.load(std::sync::atomic::Ordering::SeqCst);
        let msg = message.unwrap_or("");
        emit_progress(
            &self.app,
            &self.task_id,
            &self.phase,
            msg,
            current,
            total,
            false,
            None,
        );
    }

    fn set_position(&self, pos: u64) {
        self.current.store(pos, std::sync::atomic::Ordering::SeqCst);
        let total = self.total.load(std::sync::atomic::Ordering::SeqCst);
        emit_progress(
            &self.app,
            &self.task_id,
            &self.phase,
            "",
            pos,
            total,
            false,
            None,
        );
    }

    fn finish(&self, message: Option<&str>) {
        let total = self.total.load(std::sync::atomic::Ordering::SeqCst);
        let msg = message.unwrap_or("Complete");
        emit_progress(
            &self.app,
            &self.task_id,
            &self.phase,
            msg,
            total,
            total,
            true,
            None,
        );
    }
}

impl std::fmt::Debug for TauriProgressSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriProgressSink")
            .field("task_id", &self.task_id)
            .field("phase", &self.phase)
            .finish()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn emit_progress(
    app: &AppHandle,
    task_id: &str,
    phase: &str,
    message: &str,
    current: u64,
    total: u64,
    done: bool,
    error: Option<SerializableError>,
) {
    if task_id.starts_with("delete-") || task_id.starts_with("archive-") {
        use tauri::Manager;
        if let Some(state) = app.try_state::<crate::AppState>() {
            if let Ok(mut cleanups) = state.cleanups.lock() {
                if let Some(task) = cleanups.get_mut(task_id) {
                    task.current = current;
                    task.total = total;
                    if !message.is_empty() {
                        task.message = message.to_string();
                    }
                    if done {
                        if let Some(ref err) = error {
                            if err.code == "CANCELLED" {
                                task.status = "cancelled".to_string();
                            } else {
                                task.status = "failed".to_string();
                            }
                        } else {
                            task.status = "completed".to_string();
                        }
                    }
                }
            }
        }
    }

    let event = ClientProgressEvent {
        task_id: task_id.to_string(),
        phase: phase.to_string(),
        message: message.to_string(),
        current,
        total,
        done,
        error,
    };
    let _ = app.emit("gitpurge://progress", event);
}

// --- Submodule Declarations ---

pub mod auth;
pub mod backup;
pub mod branch;
pub mod history;
pub mod repo;
pub mod settings;
pub mod system;

#[cfg(test)]
mod tests;

// --- Public Re-exports ---

pub use auth::*;
pub use backup::*;
pub use branch::*;
pub use history::*;
pub use repo::*;
pub use settings::*;
pub use system::*;
