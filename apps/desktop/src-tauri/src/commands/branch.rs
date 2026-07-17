use std::path::Path;
use tauri::{AppHandle, State};
use tokio::sync::oneshot;

use gitpurge_core::model::{
    Action, ActionFilter, ActionKind, Activity, BranchName, BranchScope, Classification, Commit,
    ExecMode, GlobPattern, MergeState, NamingVerdict, Oid, Plan, Protection, Recommendation,
    RefBasis, RepoId, ScanOptions, Signature, TrackingFacet,
};

use super::{
    emit_progress, format_datetime, map_diff_result, map_error, map_plan, map_run_report,
    map_scan_result, map_tree_view, ActiveCleanupTask, ClientActionFilter, ClientDiffResult,
    ClientExecOptions, ClientPlan, ClientRunReport, ClientScanOptions, ClientScanResult,
    ClientTreeView, SerializableError, TauriProgressSink,
};
use crate::AppState;

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
            None
        } else {
            Some(BranchScope::Local)
        };

        let core_opts = ScanOptions {
            age_override: options.age,
            excludes: Vec::new(),
            scope,
            include_all: false,
            auto_fetch: options.auto_fetch.unwrap_or(true),
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
    let remotes = engine
        .get_remotes(&RepoId(repo_id.clone()))
        .map_err(map_error)?;

    let raw_refs = filter.refs.clone().unwrap_or_default();
    let core_filter = map_action_filter(filter, &remotes);
    let plan = engine
        .plan(&RepoId(repo_id), &core_filter)
        .map_err(map_error)?;
    let mut client_plan = map_plan(plan);

    if !raw_refs.is_empty() {
        client_plan
            .actions
            .retain(|action| raw_refs.contains(&action.ref_name));
    }

    Ok(client_plan)
}

pub fn map_action_filter(filter: ClientActionFilter, remotes: &[String]) -> ActionFilter {
    let kind = match filter.kind.as_str() {
        "archive" => ActionKind::Archive,
        _ => ActionKind::Delete,
    };
    let specific_branches = filter
        .refs
        .unwrap_or_default()
        .into_iter()
        .map(|r| {
            let mut stripped = r.as_str();
            for remote in remotes {
                let prefix = format!("{}/", remote);
                if r.starts_with(&prefix) {
                    stripped = r.strip_prefix(&prefix).unwrap();
                    break;
                }
            }
            if stripped == r.as_str() {
                stripped = r.strip_prefix("origin/").unwrap_or(&r);
            }
            BranchName(stripped.to_string())
        })
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

    {
        let mut cleanups = state.cleanups.lock().unwrap();
        if !cleanups.contains_key(&task_id) {
            cleanups.insert(
                task_id.clone(),
                ActiveCleanupTask {
                    task_id: task_id.clone(),
                    repo_id: repo_id.clone(),
                    kind: "delete".to_string(),
                    status: "running".to_string(),
                    current: 0,
                    total: plan.actions.len() as u64,
                    message: "Initializing branch deletion...".to_string(),
                    started_at: format_datetime(time::OffsetDateTime::now_utc()),
                },
            );
        }
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
                let (branch_name, remote) = if scope == BranchScope::Remote {
                    let parts: Vec<&str> = a.ref_name.splitn(2, '/').collect();
                    if parts.len() == 2 {
                        (parts[1].to_string(), Some(parts[0].to_string()))
                    } else {
                        (a.ref_name.clone(), Some("origin".to_string()))
                    }
                } else {
                    (a.ref_name.clone(), None)
                };

                let merge_state = match a.classification.merge.as_str() {
                    "merged" => MergeState::Merged,
                    _ => MergeState::Unmerged,
                };

                Action {
                    kind,
                    branch: BranchName(branch_name.clone()),
                    scope,
                    remote: remote.clone(),
                    classification: Classification {
                        branch: BranchName(branch_name),
                        scope,
                        remote,
                        upstream: None,
                        merge_state,
                        activity: Activity::Active,
                        age: std::time::Duration::default(),
                        tip: Commit {
                            oid: Oid(a.classification.locality.clone()),
                            short: String::new(),
                            author: Signature {
                                name: String::new(),
                                email: String::new(),
                                when: time::OffsetDateTime::now_utc(),
                            },
                            committer: Signature {
                                name: String::new(),
                                email: String::new(),
                                when: time::OffsetDateTime::now_utc(),
                            },
                            author_date: time::OffsetDateTime::now_utc(),
                            commit_date: time::OffsetDateTime::now_utc(),
                            subject: String::new(),
                            parents: Vec::new(),
                        },
                        tracking: TrackingFacet {
                            ahead: 0,
                            behind: 0,
                            upstream_gone: false,
                            compared_against: RefBasis::DefaultBranch,
                        },
                        protection: Protection::Unprotected,
                        naming: NamingVerdict::Standard,
                        recommendation: Recommendation::NoAction,
                    },
                    reason: a.reason.clone(),
                }
            })
            .collect();

        let core_plan = Plan {
            repo: RepoId(plan.repo_id),
            actions,
            skipped_count: 0,
            summary: String::new(),
        };

        let mode = if exec.no_backup {
            ExecMode::DryRun // fallback or handle appropriately in core
        } else {
            ExecMode::Execute
        };

        let progress_sink = TauriProgressSink::new(
            app_clone.clone(),
            task_id_clone.clone(),
            "delete".to_string(),
        );

        engine.execute_with_progress(&core_plan, mode, exec.no_backup, &progress_sink)
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
    mut plan: ClientPlan,
    exec: ClientExecOptions,
    task_id: String,
) -> Result<ClientRunReport, SerializableError> {
    {
        let mut cleanups = state.cleanups.lock().unwrap();
        cleanups.insert(
            task_id.clone(),
            ActiveCleanupTask {
                task_id: task_id.clone(),
                repo_id: repo_id.clone(),
                kind: "archive".to_string(),
                status: "running".to_string(),
                current: 0,
                total: plan.actions.len() as u64,
                message: "Initializing branch archival...".to_string(),
                started_at: format_datetime(time::OffsetDateTime::now_utc()),
            },
        );
    }

    let engine = state.engine.clone();

    // 1. Resolve target branch
    let target = exec
        .target_branch
        .clone()
        .unwrap_or_else(|| "main-legacy".to_string());

    // 2. Resolve merge strategy
    let strategy = match exec.strategy.as_deref() {
        Some("theirs") => gitpurge_core::action::ArchiveStrategy::Theirs,
        _ => gitpurge_core::action::ArchiveStrategy::Ours,
    };

    // 3. Resolve list of branches to archive (local only)
    let branches_to_archive: Vec<gitpurge_core::model::BranchName> = plan
        .actions
        .iter()
        .filter(|a| a.action == "archive" && a.classification.locality == "local")
        .map(|a| gitpurge_core::model::BranchName(a.ref_name.clone()))
        .collect();

    if !branches_to_archive.is_empty() {
        emit_progress(
            &app,
            &task_id,
            "archive",
            &format!(
                "Merging {} branches into '{}'...",
                branches_to_archive.len(),
                target
            ),
            20,
            100,
            false,
            None,
        );

        let repo_id_core = gitpurge_core::model::RepoId(repo_id.clone());

        // Run core archiving to merge them
        engine
            .archive(&repo_id_core, &branches_to_archive, &target, strategy, true)
            .map_err(map_error)?;
    }

    // 4. Change action to "delete" for these actions in the plan, so delete_branches deletes them
    for a in &mut plan.actions {
        if a.action == "archive" {
            a.action = "delete".to_string();
        }
    }

    // 5. Delegate to delete_branches to create the safety backup, delete them from git, and return the report
    delete_branches(app, state, repo_id, plan, exec, task_id).await
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

#[tauri::command]
pub async fn cancel(state: State<'_, AppState>, task_id: String) -> Result<(), SerializableError> {
    let mut tasks = state.tasks.lock().unwrap();
    if let Some(tx) = tasks.remove(&task_id) {
        let _ = tx.send(());
    }
    {
        let mut cleanups = state.cleanups.lock().unwrap();
        if let Some(task) = cleanups.get_mut(&task_id) {
            task.status = "cancelled".to_string();
            task.message = "Cancelled by user".to_string();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_active_cleanups(
    state: State<'_, AppState>,
) -> Result<Vec<ActiveCleanupTask>, SerializableError> {
    let cleanups = state.cleanups.lock().unwrap();
    Ok(cleanups.values().cloned().collect())
}
