use tauri::{AppHandle, State};
use tokio::sync::oneshot;

use gitpurge_core::model::{
    BranchName, ExecMode, RepoId, RestoreSpec, RetentionPolicy, SnapshotId, SnapshotTrigger,
};

use super::{
    emit_progress, map_error, map_prune_report, map_restore_outcome, map_snapshot,
    map_snapshot_detail, map_verify_report, ClientBackupOptions, ClientPruneReport,
    ClientRestoreOutcome, ClientRestoreSpec, ClientSnapshot, ClientSnapshotDetail,
    ClientVerifyReport, SerializableError,
};
use crate::AppState;

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
        let repo_id = match engine.get_snapshot(&snap_id) {
            Ok(Some(snap)) => snap.repo.clone(),
            _ => RepoId("default".to_string()),
        };

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
        original_ref: spec.original_ref,
    }
}
