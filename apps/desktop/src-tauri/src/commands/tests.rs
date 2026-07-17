use super::{map_repo_summary, save_file};
use crate::AppState;
use gitpurge_core::history::{HistoryStore, SqliteHistoryStore};
use gitpurge_core::model::{RepoId, Repository, RunMetrics, RunReport};
use gitpurge_core::Engine;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use time::OffsetDateTime;

#[test]
fn test_map_repo_summary_reads_history_cache() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("gitpurge-cache-test-{}", now));
    let db_path = temp_dir.join("history.db");
    let backups_root = temp_dir.join("backups");
    fs::create_dir_all(&backups_root).unwrap();

    let history = Box::new(SqliteHistoryStore::new(&db_path, backups_root).unwrap());

    let repo_id = RepoId("test-repo-cache".to_string());
    let repo = Repository {
        id: repo_id.clone(),
        display_name: "Test Repo Cache".to_string(),
        local_path: Some(PathBuf::from("/nonexistent/path")),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: OffsetDateTime::now_utc(),
        last_scanned_at: Some(OffsetDateTime::now_utc()),
    };

    // Save repo and a run with metrics
    history.save_repo(&repo).unwrap();
    let run_report = RunReport {
        id: "run-1".to_string(),
        repo: repo_id.clone(),
        command: "scan".to_string(),
        mode: gitpurge_core::model::ExecMode::DryRun,
        started_at: OffsetDateTime::now_utc(),
        snapshot: None,
        results: Vec::new(),
        success_count: 0,
        failure_count: 0,
        skipped_count: 0,
        metrics: Some(RunMetrics {
            total: 42,
            active: 20,
            stale: 10,
            merged: 5,
            unmerged: 7,
            non_standard: 0,
            local_count: Some(40),
            remote_count: Some(2),
            protected: Some(8),
            deleted: None,
            archived: None,
            restored: None,
        }),
        branch_snapshots: None,
    };
    history.record_run(&run_report).unwrap();

    // Construct Engine
    let config = gitpurge_core::Config::default();
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(gitpurge_core::clock::SystemClock);
    let progress = Box::new(gitpurge_core::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    // Call map_repo_summary. Since history has entries, it should read from the database
    // and return the cached counts without trying to scan the nonexistent path.
    let summary = map_repo_summary(&engine, &repo);
    assert_eq!(summary.branch_count, 42);
    assert_eq!(summary.stale, 10);
    assert_eq!(summary.unmerged, 7);
    assert_eq!(summary.protected_count, 8);

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_save_file_command() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("gitpurge-save-test-{}", now));
    fs::create_dir_all(&temp_dir).unwrap();

    let file_path = temp_dir.join("test_write.txt");
    let path_str = file_path.to_string_lossy().to_string();
    let res = save_file(path_str.clone(), "Hello World".to_string());
    assert!(res.is_ok());
    let read_back = fs::read_to_string(&file_path).unwrap();
    assert_eq!(read_back, "Hello World");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_desktop_tauri_commands() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("gitpurge-cmd-test-{}", now));
        let db_path = temp_dir.join("history.db");
        let backups_root = temp_dir.join("backups");
        fs::create_dir_all(&backups_root).unwrap();

        let history = Box::new(SqliteHistoryStore::new(&db_path, backups_root).unwrap());
        let config = gitpurge_core::Config::default();
        let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
        let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
        let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
        let clock = Box::new(gitpurge_core::clock::SystemClock);
        let progress = Box::new(gitpurge_core::progress::NoopProgressSink);

        let engine = Engine::new(
            config,
            git_backend,
            secrets,
            history,
            report_sink,
            clock,
            progress,
        );

        let app = tauri::test::mock_app();
        let app_state = AppState {
            engine: Arc::new(engine),
            tasks: Mutex::new(HashMap::new()),
            cleanups: Mutex::new(HashMap::new()),
        };
        app.manage(app_state);

        let state = app.state::<AppState>();

        // Test settings_get
        let initial_settings = super::settings_get(state.clone()).await.unwrap();
        assert_eq!(initial_settings.date_format, "YYYY-MM-DD h:m a");

        // Test settings_save
        let mut new_settings = initial_settings.clone();
        new_settings.date_format = "YYYY-MM-DD".to_string();
        let saved_settings = super::settings_save(state.clone(), new_settings)
            .await
            .unwrap();
        assert_eq!(saved_settings.date_format, "YYYY-MM-DD");

        // Verify it saved to config
        let get_again = super::settings_get(state.clone()).await.unwrap();
        assert_eq!(get_again.date_format, "YYYY-MM-DD");

        // Test repo_list (should be empty initially)
        let repos = super::repo_list(state.clone()).await.unwrap();
        assert!(repos.is_empty());

        // Test repo_add
        let path_str = temp_dir.to_string_lossy().to_string();
        let repo_summary = super::repo_add(
            state.clone(),
            Some(path_str),
            None,
            Some("Test Repository".to_string()),
        )
        .await
        .unwrap();
        assert_eq!(repo_summary.name, "Test Repository");

        // Test repo_list again (should contain 1)
        let repos = super::repo_list(state.clone()).await.unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "Test Repository");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    });
}

#[test]
fn test_desktop_backup_commands() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("gitpurge-backup-test-{}", now));
        let db_path = temp_dir.join("history.db");
        let backups_root = temp_dir.join("backups");
        fs::create_dir_all(&backups_root).unwrap();

        let history = Box::new(SqliteHistoryStore::new(&db_path, backups_root).unwrap());
        let config = gitpurge_core::Config::default();
        let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
        let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
        let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
        let clock = Box::new(gitpurge_core::clock::SystemClock);
        let progress = Box::new(gitpurge_core::progress::NoopProgressSink);

        let engine = Engine::new(
            config,
            git_backend,
            secrets,
            history,
            report_sink,
            clock,
            progress,
        );

        let app = tauri::test::mock_app();
        let app_state = AppState {
            engine: Arc::new(engine),
            tasks: Mutex::new(HashMap::new()),
            cleanups: Mutex::new(HashMap::new()),
        };
        app.manage(app_state);

        let state = app.state::<AppState>();

        // Test backup_list is empty
        let snaps = super::backup_list(state.clone(), "test-repo".to_string())
            .await
            .unwrap();
        assert!(snaps.is_empty());

        // Test backup_prune (should run successfully even if empty)
        let prune_rep = super::backup_prune(state.clone(), "test-repo".to_string(), Some(5), None)
            .await
            .unwrap();
        assert_eq!(prune_rep.removed.len(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    });
}
