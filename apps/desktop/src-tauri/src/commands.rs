use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;

use gitpurge_core::{
    model::{
        ActionFilter, ActionKind, ActionResult, BranchName, BranchScope, ExecMode, GlobPattern,
        Oid, Plan, RepoId, Repository, RestoreSpec, RetentionPolicy, RunReport, ScanOptions,
        ScanResult, Snapshot, SnapshotId, SnapshotTrigger,
    },
    Config, Engine, GitPurgeError,
};

use crate::AppState;

// --- TS/IPC Projections ---

#[derive(serde::Serialize, Clone)]
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
    let counts = if let Ok(scan_res) = engine.scan(&repo.id, ScanOptions::default()) {
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
            .filter(|c| !matches!(c.protection, gitpurge_core::model::Protection::Unprotected))
            .count();
        (scan_res.total_branches, stale, unmerged, protected)
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
        last_scanned: repo.last_scanned_at.map(|t| t.to_string()),
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
        .map(|c| Branch {
            name: c.branch.0.clone(),
            ref_path: format!("refs/heads/{}", c.branch.0),
            tip_sha: c.tip.oid.0.clone(),
            tip_short: c.tip.short.clone(),
            author_name: c.tip.author.name.clone(),
            committed_at: c.tip.commit_date.to_string(),
            age_days: c.age.as_secs() / 86400,
            upstream: c.branch.0.clone().into(),
            classification: map_classification(&c),
        })
        .collect();

    ClientScanResult {
        repo_id: scan_res.repo.0,
        scanned_at: time::OffsetDateTime::now_utc().to_string(),
        branches,
    }
}

pub fn map_plan(core_plan: Plan) -> ClientPlan {
    let actions = core_plan
        .actions
        .into_iter()
        .map(|a| ClientPlannedAction {
            ref_name: a.branch.0.clone(),
            action: match a.kind {
                gitpurge_core::model::ActionKind::Delete => "delete",
                gitpurge_core::model::ActionKind::Archive => "archive",
                _ => "delete",
            }
            .to_string(),
            reason: a.reason,
            classification: map_classification(&a.classification),
            destructive: a.classification.merge_state == gitpurge_core::model::MergeState::Unmerged,
        })
        .collect();

    ClientPlan {
        repo_id: core_plan.repo.0,
        kind: "delete".to_string(),
        actions,
        created_at: time::OffsetDateTime::now_utc().to_string(),
    }
}

pub fn map_run_report(core_report: RunReport) -> ClientRunReport {
    let per_ref = core_report
        .results
        .into_iter()
        .map(|r| {
            let (ref_name, outcome, error) = match r {
                ActionResult::Success { action, .. } => {
                    (action.branch.0.clone(), "done".to_string(), None)
                }
                ActionResult::Failed { action, error } => (
                    action.branch.0.clone(),
                    "failed".to_string(),
                    Some(SerializableError {
                        code: "GIT_ERROR".to_string(),
                        message: error.clone(),
                        hint: None,
                    }),
                ),
                ActionResult::Skipped { action } => {
                    (action.branch.0.clone(), "skipped".to_string(), None)
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
        started_at: time::OffsetDateTime::now_utc().to_string(),
        finished_at: time::OffsetDateTime::now_utc().to_string(),
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
        created_at: core_snap.created_at.to_string(),
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
        .map(|r| ClientSnapshotRef {
            branch: r.branch.0.clone(),
            tip_sha: r.tip.0.clone(),
            commit_count: r.commit_count as usize,
            upstream: r.upstream.clone(),
            merge: match r.merged_at_capture {
                gitpurge_core::model::MergeState::Merged => "merged",
                _ => "unmerged",
            }
            .to_string(),
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

pub fn map_diff_result(
    repo_id: &str,
    core_res: gitpurge_core::diff::DiffResult,
) -> ClientDiffResult {
    let files = core_res
        .entries
        .into_iter()
        .map(|e| DiffFile {
            path: e.path,
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
    }
}

// --- Progress Event Emitter helper ---

#[allow(clippy::too_many_arguments)]
fn emit_progress(
    app: &AppHandle,
    task_id: &str,
    phase: &str,
    message: &str,
    current: u64,
    total: u64,
    done: bool,
    error: Option<SerializableError>,
) {
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

// --- Tauri Commands ---

#[tauri::command]
pub async fn repo_list(state: State<'_, AppState>) -> Result<Vec<RepoSummary>, SerializableError> {
    let engine = &state.engine;
    let repos = engine.list_repos().map_err(map_error)?;
    let summaries = repos
        .into_iter()
        .map(|r| map_repo_summary(engine, &r))
        .collect();
    Ok(summaries)
}

#[tauri::command]
pub async fn repo_add(
    state: State<'_, AppState>,
    path: Option<String>,
    url: Option<String>,
    name: Option<String>,
) -> Result<RepoSummary, SerializableError> {
    let engine = &state.engine;
    let repo = if let Some(p) = path {
        Repository::new_local(PathBuf::from(p)).map_err(map_error)?
    } else if let Some(u) = url {
        let git_url = gitpurge_core::model::GitUrl::parse(&u).map_err(map_error)?;
        Repository::new_remote(git_url).map_err(map_error)?
    } else {
        return Err(SerializableError {
            code: "CONFIG".to_string(),
            message: "Either path or remote url must be supplied to add repository.".to_string(),
            hint: None,
        });
    };

    let mut repo = repo;
    if let Some(n) = name {
        repo.display_name = n;
    }

    engine.add_repo(repo.clone()).map_err(map_error)?;
    let _ = engine.save_config(None);
    Ok(map_repo_summary(engine, &repo))
}

#[tauri::command]
pub async fn repo_remove(
    state: State<'_, AppState>,
    repo_id: String,
    drop_backups: Option<bool>,
) -> Result<(), SerializableError> {
    let engine = &state.engine;
    let id = RepoId(repo_id);
    if drop_backups.unwrap_or(false) {
        let _ = engine.purge_repo_backups(&id);
    }
    engine.remove_repo(&id).map_err(map_error)?;
    let _ = engine.save_config(None);
    Ok(())
}

#[tauri::command]
pub async fn repo_show(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<RepoDetail, SerializableError> {
    let engine = &state.engine;
    let id = RepoId(repo_id);
    let repo = engine
        .get_repo(&id)
        .map_err(map_error)?
        .ok_or_else(|| SerializableError {
            code: "NOT_FOUND".to_string(),
            message: format!("Repository not found: {}", id.0),
            hint: None,
        })?;
    Ok(map_repo_detail(engine, &repo))
}

#[tauri::command]
pub async fn scan(
    app: AppHandle,
    state: State<'_, AppState>,
    repo_id: String,
    options: ClientScanOptions,
    task_id: String,
) -> Result<ClientScanResult, SerializableError> {
    let (tx, rx) = oneshot::channel::<()>();
    {
        let mut tasks = state.tasks.lock().unwrap();
        tasks.insert(task_id.clone(), tx);
    }

    let engine = state.engine.clone();
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();

    let work = tokio::task::spawn_blocking(move || {
        emit_progress(
            &app_clone,
            &task_id_clone,
            "scan",
            "Starting scan...",
            0,
            100,
            false,
            None,
        );

        let scope = if options.include_remote.unwrap_or(true) {
            BranchScope::Remote
        } else {
            BranchScope::Local
        };

        let core_opts = ScanOptions {
            age_override: options.age,
            excludes: Vec::new(),
            scope: Some(scope),
            include_all: false,
        };

        emit_progress(
            &app_clone,
            &task_id_clone,
            "scan",
            "Analyzing repository branches...",
            40,
            100,
            false,
            None,
        );

        let res = engine.scan(&RepoId(repo_id), core_opts);

        emit_progress(
            &app_clone,
            &task_id_clone,
            "scan",
            "Finishing scan...",
            95,
            100,
            false,
            None,
        );

        res
    });

    tokio::select! {
        res = work => {
            {
                let mut tasks = state.tasks.lock().unwrap();
                tasks.remove(&task_id);
            }
            let core_res = res.map_err(|e| SerializableError {
                code: "GENERIC".to_string(),
                message: format!("Thread join failed: {}", e),
                hint: None,
            })?;
            match core_res {
                Ok(scan_result) => {
                    emit_progress(&app, &task_id, "scan", "Scan complete.", 100, 100, true, None);
                    Ok(map_scan_result(scan_result))
                }
                Err(err) => {
                    let ser_err = map_error(err);
                    emit_progress(&app, &task_id, "scan", "Scan failed", 100, 100, true, Some(ser_err.clone()));
                    Err(ser_err)
                }
            }
        }
        _ = rx => {
            emit_progress(&app, &task_id, "scan", "Scan cancelled", 100, 100, true, Some(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            }));
            Err(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            })
        }
    }
}

#[tauri::command]
pub async fn plan(
    state: State<'_, AppState>,
    repo_id: String,
    filter: ClientActionFilter,
) -> Result<ClientPlan, SerializableError> {
    let engine = &state.engine;
    let core_filter = map_action_filter(filter);
    let plan = engine
        .plan(&RepoId(repo_id), &core_filter)
        .map_err(map_error)?;
    Ok(map_plan(plan))
}

pub fn map_action_filter(filter: ClientActionFilter) -> ActionFilter {
    let kind = match filter.kind.as_str() {
        "archive" => ActionKind::Archive,
        _ => ActionKind::Delete,
    };
    let specific_branches = filter
        .refs
        .unwrap_or_default()
        .into_iter()
        .map(BranchName)
        .collect();
    let exclude_globs = filter
        .exclude
        .unwrap_or_default()
        .into_iter()
        .map(GlobPattern)
        .collect();

    ActionFilter {
        kind: Some(kind),
        age_override: filter.age,
        merged_only: filter.merged.unwrap_or(false),
        include_unmerged: filter.include_unmerged.unwrap_or(false),
        specific_branches,
        include_globs: Vec::new(),
        exclude_globs,
    }
}

#[tauri::command]
pub async fn backup_create(
    app: AppHandle,
    state: State<'_, AppState>,
    repo_id: String,
    options: ClientBackupOptions,
    task_id: String,
) -> Result<ClientSnapshot, SerializableError> {
    let (tx, rx) = oneshot::channel::<()>();
    {
        let mut tasks = state.tasks.lock().unwrap();
        tasks.insert(task_id.clone(), tx);
    }

    let engine = state.engine.clone();
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();

    let work = tokio::task::spawn_blocking(move || {
        emit_progress(
            &app_clone,
            &task_id_clone,
            "backup",
            "Initializing backup snapshot...",
            10,
            100,
            false,
            None,
        );

        let core_opts = map_backup_options(options);

        emit_progress(
            &app_clone,
            &task_id_clone,
            "backup",
            "Writing snapshot references to mirror...",
            50,
            100,
            false,
            None,
        );

        let res = engine.backup_create(&RepoId(repo_id), core_opts);

        emit_progress(
            &app_clone,
            &task_id_clone,
            "backup",
            "Verifying backup integrity...",
            85,
            100,
            false,
            None,
        );

        res
    });

    tokio::select! {
        res = work => {
            {
                let mut tasks = state.tasks.lock().unwrap();
                tasks.remove(&task_id);
            }
            let core_res = res.map_err(|e| SerializableError {
                code: "GENERIC".to_string(),
                message: format!("Thread join failed: {}", e),
                hint: None,
            })?;
            match core_res {
                Ok(snapshot) => {
                    emit_progress(&app, &task_id, "backup", "Backup created successfully.", 100, 100, true, None);
                    Ok(map_snapshot(&snapshot))
                }
                Err(err) => {
                    let ser_err = map_error(err);
                    emit_progress(&app, &task_id, "backup", "Backup failed", 100, 100, true, Some(ser_err.clone()));
                    Err(ser_err)
                }
            }
        }
        _ = rx => {
            emit_progress(&app, &task_id, "backup", "Backup cancelled", 100, 100, true, Some(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            }));
            Err(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            })
        }
    }
}

pub fn map_backup_options(opts: ClientBackupOptions) -> gitpurge_core::model::BackupOptions {
    let trigger = match opts.trigger.as_str() {
        "preDelete" => SnapshotTrigger::PreDelete,
        "scheduled" => SnapshotTrigger::Scheduled,
        _ => SnapshotTrigger::Manual,
    };
    let only_branches = opts
        .refs
        .unwrap_or_default()
        .into_iter()
        .map(BranchName)
        .collect();

    gitpurge_core::model::BackupOptions {
        trigger: Some(trigger),
        verify: opts.verify,
        only_branches,
    }
}

#[tauri::command]
pub async fn backup_list(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<Vec<ClientSnapshot>, SerializableError> {
    let engine = &state.engine;
    let list = engine.list_snapshots(&RepoId(repo_id)).map_err(map_error)?;
    let mapped = list.iter().map(map_snapshot).collect();
    Ok(mapped)
}

#[tauri::command]
pub async fn backup_show(
    state: State<'_, AppState>,
    snapshot_id: String,
) -> Result<ClientSnapshotDetail, SerializableError> {
    let engine = &state.engine;
    let id = SnapshotId(snapshot_id);
    let snap = engine
        .get_snapshot(&id)
        .map_err(map_error)?
        .ok_or_else(|| SerializableError {
            code: "NOT_FOUND".to_string(),
            message: format!("Snapshot not found: {}", id.0),
            hint: None,
        })?;
    Ok(map_snapshot_detail(&snap))
}

#[tauri::command]
pub async fn backup_verify(
    app: AppHandle,
    state: State<'_, AppState>,
    snapshot_id: String,
    task_id: String,
) -> Result<ClientVerifyReport, SerializableError> {
    let (tx, rx) = oneshot::channel::<()>();
    {
        let mut tasks = state.tasks.lock().unwrap();
        tasks.insert(task_id.clone(), tx);
    }

    let engine = state.engine.clone();
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();

    let work = tokio::task::spawn_blocking(move || {
        emit_progress(
            &app_clone,
            &task_id_clone,
            "verify",
            "Checking snapshot references...",
            20,
            100,
            false,
            None,
        );

        let snap_id = SnapshotId(snapshot_id);
        let repo_id = RepoId("default".to_string()); // fallback or resolve if needed

        emit_progress(
            &app_clone,
            &task_id_clone,
            "verify",
            "Probing commit objects in mirror...",
            60,
            100,
            false,
            None,
        );

        let res = engine.backup_verify(&repo_id, &snap_id);

        emit_progress(
            &app_clone,
            &task_id_clone,
            "verify",
            "Finalizing verification...",
            90,
            100,
            false,
            None,
        );

        res
    });

    tokio::select! {
        res = work => {
            {
                let mut tasks = state.tasks.lock().unwrap();
                tasks.remove(&task_id);
            }
            let core_res = res.map_err(|e| SerializableError {
                code: "GENERIC".to_string(),
                message: format!("Thread join failed: {}", e),
                hint: None,
            })?;
            match core_res {
                Ok(report) => {
                    emit_progress(&app, &task_id, "verify", "Verification complete.", 100, 100, true, None);
                    Ok(map_verify_report(report))
                }
                Err(err) => {
                    let ser_err = map_error(err);
                    emit_progress(&app, &task_id, "verify", "Verification failed", 100, 100, true, Some(ser_err.clone()));
                    Err(ser_err)
                }
            }
        }
        _ = rx => {
            emit_progress(&app, &task_id, "verify", "Verification cancelled", 100, 100, true, Some(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            }));
            Err(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            })
        }
    }
}

#[tauri::command]
pub async fn backup_prune(
    state: State<'_, AppState>,
    repo_id: String,
    keep: Option<usize>,
    older_than: Option<String>,
) -> Result<ClientPruneReport, SerializableError> {
    let engine = &state.engine;
    let mut policy = RetentionPolicy::default();
    if let Some(k) = keep {
        policy.keep_last = Some(k);
    }
    if let Some(ref o) = older_than {
        // Simple age conversion (stubbed / fallback as dur)
        if o == "1 year ago" {
            policy.keep_within = Some(std::time::Duration::from_secs(31536000));
        }
    }

    let report = engine
        .backup_prune(&RepoId(repo_id), &policy, ExecMode::Execute)
        .map_err(map_error)?;
    Ok(map_prune_report(report))
}

#[tauri::command]
pub async fn delete_branches(
    app: AppHandle,
    state: State<'_, AppState>,
    repo_id: String,
    plan: ClientPlan,
    exec: ClientExecOptions,
    task_id: String,
) -> Result<ClientRunReport, SerializableError> {
    let (tx, rx) = oneshot::channel::<()>();
    {
        let mut tasks = state.tasks.lock().unwrap();
        tasks.insert(task_id.clone(), tx);
    }

    let engine = state.engine.clone();
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();

    let work = tokio::task::spawn_blocking(move || {
        emit_progress(
            &app_clone,
            &task_id_clone,
            "delete",
            "Preparing execution plan...",
            10,
            100,
            false,
            None,
        );

        // Convert client plan back to core plan
        let actions = plan
            .actions
            .iter()
            .map(|a| {
                let kind = match a.action.as_str() {
                    "archive" => ActionKind::Archive,
                    _ => ActionKind::Delete,
                };
                let scope = match a.classification.locality.as_str() {
                    "remote" => BranchScope::Remote,
                    _ => BranchScope::Local,
                };
                let remote = if scope == BranchScope::Remote {
                    Some("origin".to_string())
                } else {
                    None
                };

                let merge_state = match a.classification.merge.as_str() {
                    "merged" => gitpurge_core::model::MergeState::Merged,
                    _ => gitpurge_core::model::MergeState::Unmerged,
                };

                gitpurge_core::model::Action {
                    kind,
                    branch: BranchName(a.ref_name.clone()),
                    scope,
                    remote,
                    classification: gitpurge_core::model::Classification {
                        branch: BranchName(a.ref_name.clone()),
                        scope,
                        merge_state,
                        activity: gitpurge_core::model::Activity::Active,
                        age: std::time::Duration::default(),
                        protection: gitpurge_core::model::Protection::Unprotected,
                        naming: gitpurge_core::model::NamingVerdict::Standard,
                        tracking: gitpurge_core::model::TrackingFacet {
                            ahead: 0,
                            behind: 0,
                            upstream_gone: false,
                            compared_against: gitpurge_core::model::RefBasis::DefaultBranch,
                        },
                        tip: gitpurge_core::model::Commit {
                            oid: Oid(String::new()),
                            short: String::new(),
                            author: gitpurge_core::model::Signature {
                                name: String::new(),
                                email: String::new(),
                                when: time::OffsetDateTime::now_utc(),
                            },
                            committer: gitpurge_core::model::Signature {
                                name: String::new(),
                                email: String::new(),
                                when: time::OffsetDateTime::now_utc(),
                            },
                            author_date: time::OffsetDateTime::now_utc(),
                            commit_date: time::OffsetDateTime::now_utc(),
                            subject: String::new(),
                            parents: Vec::new(),
                        },
                        recommendation: gitpurge_core::model::Recommendation::DeleteMerged,
                    },
                    reason: a.reason.clone(),
                }
            })
            .collect();

        let core_plan = Plan {
            repo: RepoId(repo_id),
            actions,
            skipped_count: 0,
            summary: String::new(),
        };

        if !exec.no_backup {
            emit_progress(
                &app_clone,
                &task_id_clone,
                "delete",
                "Creating pre-delete safety backup...",
                30,
                100,
                false,
                None,
            );
        }

        emit_progress(
            &app_clone,
            &task_id_clone,
            "delete",
            "Executing branch deletions...",
            60,
            100,
            false,
            None,
        );

        let res = engine.execute(&core_plan, ExecMode::Execute, exec.no_backup);

        emit_progress(
            &app_clone,
            &task_id_clone,
            "delete",
            "Verifying outcomes...",
            90,
            100,
            false,
            None,
        );

        res
    });

    tokio::select! {
        res = work => {
            {
                let mut tasks = state.tasks.lock().unwrap();
                tasks.remove(&task_id);
            }
            let core_res = res.map_err(|e| SerializableError {
                code: "GENERIC".to_string(),
                message: format!("Thread join failed: {}", e),
                hint: None,
            })?;
            match core_res {
                Ok(report) => {
                    emit_progress(&app, &task_id, "delete", "Deletion complete.", 100, 100, true, None);
                    Ok(map_run_report(report))
                }
                Err(err) => {
                    let ser_err = map_error(err);
                    emit_progress(&app, &task_id, "delete", "Deletion failed", 100, 100, true, Some(ser_err.clone()));
                    Err(ser_err)
                }
            }
        }
        _ = rx => {
            emit_progress(&app, &task_id, "delete", "Deletion cancelled", 100, 100, true, Some(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            }));
            Err(SerializableError {
                code: "CANCELLED".to_string(),
                message: "Operation was cancelled by user".to_string(),
                hint: None,
            })
        }
    }
}

#[tauri::command]
pub async fn archive_branches(
    app: AppHandle,
    state: State<'_, AppState>,
    repo_id: String,
    plan: ClientPlan,
    exec: ClientExecOptions,
    task_id: String,
) -> Result<ClientRunReport, SerializableError> {
    // Thinly route to delete_branches as they share the engine.execute route
    delete_branches(app, state, repo_id, plan, exec, task_id).await
}

#[tauri::command]
pub async fn restore(
    state: State<'_, AppState>,
    snapshot_id: String,
    spec: ClientRestoreSpec,
) -> Result<ClientRestoreOutcome, SerializableError> {
    let engine = &state.engine;
    let core_spec = map_restore_spec(spec);
    let outcome = engine
        .restore(&SnapshotId(snapshot_id), core_spec)
        .map_err(map_error)?;
    Ok(map_restore_outcome(outcome))
}

pub fn map_restore_spec(spec: ClientRestoreSpec) -> RestoreSpec {
    RestoreSpec {
        branch: BranchName(spec.ref_name),
        as_tag: spec.target_type == "tag",
        target_name: spec.new_name,
        force: spec.force,
    }
}

#[tauri::command]
pub async fn diff(
    state: State<'_, AppState>,
    repo_id: String,
    a: String,
    b: String,
) -> Result<ClientDiffResult, SerializableError> {
    let engine = &state.engine;
    let spec_a = gitpurge_core::model::RefSpec::Branch(BranchName(a));
    let spec_b = gitpurge_core::model::RefSpec::Branch(BranchName(b));
    let diff = engine
        .diff(&RepoId(repo_id.clone()), &spec_a, &spec_b)
        .map_err(map_error)?;
    Ok(map_diff_result(&repo_id, diff))
}

#[tauri::command]
pub async fn show_tree(
    state: State<'_, AppState>,
    repo_id: String,
    at: String,
    path: Option<String>,
) -> Result<ClientTreeView, SerializableError> {
    let engine = &state.engine;
    let ref_spec = gitpurge_core::model::RefSpec::Branch(BranchName(at));
    let path_buf = path.as_ref().map(Path::new);
    let tree = engine
        .show_tree(&RepoId(repo_id.clone()), &ref_spec, path_buf)
        .map_err(map_error)?;

    let mut blob_text = None;
    if let Some(ref p) = path {
        if let Ok(content) = engine.show_file(&RepoId(repo_id.clone()), &ref_spec, Path::new(p)) {
            blob_text = Some(String::from_utf8_lossy(&content).to_string());
        }
    }

    Ok(map_tree_view(&repo_id, tree, blob_text))
}

// --- Stubs for Phase 5 / 6 / packaging ---

#[tauri::command]
pub async fn report_generate(
    state: State<'_, AppState>,
    repo_id: String,
    format: String,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (state, repo_id, format);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Reports command not yet implemented (Phase P5)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn history_get(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (state, repo_id);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "History command not yet implemented (Phase P5)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn auth_add(
    state: State<'_, AppState>,
    credential: serde_json::Value,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (state, credential);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn auth_list(state: State<'_, AppState>) -> Result<serde_json::Value, SerializableError> {
    let _ = state;
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn auth_remove(state: State<'_, AppState>, id: String) -> Result<(), SerializableError> {
    let _ = (state, id);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn auth_test(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    repo_id: Option<String>,
    task_id: String,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (app, state, id, repo_id, task_id);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<Settings, SerializableError> {
    let config = state.engine.config();
    Ok(map_settings(&config))
}

#[tauri::command]
pub async fn settings_save(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, SerializableError> {
    let engine = &state.engine;
    let mut config = engine.config();

    // Map fields back
    config.default_policy.age = settings.policy.age;
    if !settings.policy.naming_regex.is_empty() {
        config.default_policy.naming.allowed = vec![gitpurge_core::model::RegexSource(
            settings.policy.naming_regex.clone(),
        )];
    } else {
        config.default_policy.naming.allowed = Vec::new();
    }

    config.default_policy.protection.protected_globs = settings
        .policy
        .protected_refs
        .iter()
        .map(|g| GlobPattern(g.clone()))
        .collect();

    config.default_policy.excludes = settings
        .policy
        .exclude_globs
        .iter()
        .map(|g| GlobPattern(g.clone()))
        .collect();

    if !settings.backups_root.is_empty() {
        config.backups_root = Some(PathBuf::from(settings.backups_root));
    } else {
        config.backups_root = None;
    }

    // Save to engine and disk
    engine.update_config(config.clone());
    engine.save_config(None).map_err(map_error)?;

    Ok(map_settings(&config))
}

#[tauri::command]
pub async fn install_cli(
    state: State<'_, AppState>,
    scope: String,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (state, scope);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Install CLI not yet implemented".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn cancel(state: State<'_, AppState>, task_id: String) -> Result<(), SerializableError> {
    let mut tasks = state.tasks.lock().unwrap();
    if let Some(tx) = tasks.remove(&task_id) {
        let _ = tx.send(());
    }
    Ok(())
}
