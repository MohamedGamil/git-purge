//! Round-trip integration tests for the Engine public API (P9-T2).

use gitpurge_core::{
    clock::FakeClock,
    model::{
        ActionFilter, BackupOptions, BranchName, ExecMode, RepoId, Repository, RestoreSpec,
        ScanOptions,
    },
    testkit, Config, Engine,
};

fn test_engine(config: Config) -> Engine {
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = Box::new(gitpurge_core::history::FakeHistoryStore::new());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(gitpurge_core::progress::NoopProgressSink);

    Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    )
}

#[test]
fn integration_scan_plan_backup_execute_restore_roundtrip() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("integration-repo".to_string());

    let data_root = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: Some(data_root.path().to_path_buf()),
        backups_root: Some(data_root.path().join("backups")),
        ..Default::default()
    };
    let engine = test_engine(config);

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "integration-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    let scan = engine
        .scan(
            &repo_id,
            ScanOptions {
                auto_fetch: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        scan.classifications
            .iter()
            .any(|c| c.branch.0 == "merged-branch"),
        "scan should classify merged-branch"
    );

    let filter = ActionFilter {
        merged_only: true,
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();
    assert!(
        plan.actions.iter().any(|a| a.branch.0 == "merged-branch"),
        "plan should include merged-branch delete action"
    );

    let snapshot = engine
        .backup_create(&repo_id, BackupOptions::default())
        .unwrap();
    assert!(
        !snapshot.refs.is_empty(),
        "backup snapshot should capture refs"
    );

    let report = engine.execute(&plan, ExecMode::Execute, false).unwrap();
    assert_eq!(report.success_count, 1);

    let source_repo = git2::Repository::open(repo_fixture.path()).unwrap();
    assert!(source_repo
        .find_branch("merged-branch", git2::BranchType::Local)
        .is_err());

    let outcome = engine
        .restore(
            &snapshot.id,
            RestoreSpec {
                branch: BranchName("merged-branch".to_string()),
                as_tag: false,
                target_name: None,
                force: false,
                original_ref: None,
            },
        )
        .unwrap();
    assert_eq!(outcome.created_ref, "refs/heads/merged-branch");

    assert!(source_repo
        .find_branch("merged-branch", git2::BranchType::Local)
        .is_ok());
}
