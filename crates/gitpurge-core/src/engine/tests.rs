use crate::clock::FakeClock;
use crate::config::Config;
use crate::model::{
    ActionFilter, ActionKind, BackupOptions, BranchName, GlobPattern, Policy, RefSpec, RepoId,
    Repository, RestoreSpec, ScanOptions,
};
use crate::testkit;
use crate::Engine;

#[test]
fn test_engine_scan_and_plan_flow() {
    let repo_fixture = testkit::merged_repo();

    let mut policy = Policy::default();
    policy
        .protection
        .protected_globs
        .push(GlobPattern("release/*".to_string()));

    let config = Config {
        backups_root: None,
        default_policy: policy,
        protected: Vec::new(),
        ..Default::default()
    };

    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::default());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_id = RepoId("test-repo".to_string());
    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };

    engine.register_repo(repo_model).unwrap();

    // 1. Scan
    let scan_res = engine.scan(&repo_id, ScanOptions::default()).unwrap();
    assert_eq!(scan_res.total_branches, 3);

    // 2. Plan (dry-run)
    let filter = ActionFilter {
        merged_only: true,
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();

    // main is protected, unmerged-branch is unmerged (so excluded by default merged_only filter)
    // Only merged-branch should have a delete action proposed!
    assert_eq!(plan.actions.len(), 1);
    assert_eq!(plan.actions[0].branch.0, "merged-branch");
    assert_eq!(plan.actions[0].kind, ActionKind::Delete);

    // Test diff
    let diff_res = engine
        .diff(
            &repo_id,
            &RefSpec::Branch(BranchName("main".to_string())),
            &RefSpec::Branch(BranchName("unmerged-branch".to_string())),
        )
        .unwrap();
    assert!(diff_res.files_changed > 0);

    // Test show_tree
    let tree_res = engine
        .show_tree(
            &repo_id,
            &RefSpec::Branch(BranchName("main".to_string())),
            None,
        )
        .unwrap();
    assert!(tree_res.entries.iter().any(|e| e.path == "file.txt"));
}

#[test]
fn test_backup_create_and_verify() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("test-backup-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::new());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-backup-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // 1. Create backup
    let opts = BackupOptions {
        trigger: Some(crate::model::SnapshotTrigger::Manual),
        verify: true,
        only_branches: Vec::new(),
    };
    let snapshot = engine.backup_create(&repo_id, opts).unwrap();
    assert_eq!(snapshot.refs.len(), 3); // main, merged-branch, unmerged-branch
    assert!(snapshot.verified);

    // 2. Verify snapshot
    let verify_res = crate::backup::verify_snapshot(
        &engine.config.lock().unwrap(),
        &repo_id,
        &snapshot.id,
        false,
    )
    .unwrap();
    assert!(verify_res.ok);

    // 3. Corrupt snapshot by deleting a backup reference in the mirror
    let mirror_manager = crate::backup::BackupMirrorManager::new(&engine.config.lock().unwrap());
    let mirror_path = mirror_manager.resolve_mirror_path(&repo_id);
    let mirror_repo = git2::Repository::open_bare(&mirror_path).unwrap();

    let target_ref = format!(
        "refs/gitpurge/backups/{}/refs/heads/merged-branch",
        snapshot.id.0
    );
    let mut r = mirror_repo.find_reference(&target_ref).unwrap();
    r.delete().unwrap();

    // 4. Verify snapshot should now detect corruption
    let verify_res2 = crate::backup::verify_snapshot(
        &engine.config.lock().unwrap(),
        &repo_id,
        &snapshot.id,
        false,
    )
    .unwrap();
    assert!(!verify_res2.ok);
    assert!(verify_res2
        .problems
        .contains(&crate::backup::VerifyProblem::MissingRef));
}

#[test]
fn test_restore_safeties() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("test-restore-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::new());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-restore-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // 1. Create backup snapshot
    let snapshot = engine
        .backup_create(&repo_id, BackupOptions::default())
        .unwrap();

    // 2. Delete branch in source repository
    let source_repo = git2::Repository::open(repo_fixture.path()).unwrap();
    let mut r = source_repo
        .find_reference("refs/heads/merged-branch")
        .unwrap();
    let original_oid = r.target().unwrap();
    r.delete().unwrap();
    assert!(source_repo
        .find_reference("refs/heads/merged-branch")
        .is_err());

    // 3. Restore branch
    let spec = RestoreSpec {
        branch: BranchName("merged-branch".to_string()),
        as_tag: false,
        target_name: None,
        force: false,
        original_ref: None,
    };
    let outcome = engine.restore(&snapshot.id, spec).unwrap();
    assert_eq!(outcome.created_ref, "refs/heads/merged-branch");
    assert_eq!(outcome.tip.0, original_oid.to_string());

    // 4. Verify branch is restored in source repo
    let restored_ref = source_repo
        .find_reference("refs/heads/merged-branch")
        .unwrap();
    assert_eq!(restored_ref.target().unwrap(), original_oid);

    // 5. SAFE-06: Restore again without force should fail
    let spec_no_force = RestoreSpec {
        branch: BranchName("merged-branch".to_string()),
        as_tag: false,
        target_name: None,
        force: false,
        original_ref: None,
    };
    let err = engine.restore(&snapshot.id, spec_no_force);
    assert!(err.is_err());
    assert!(matches!(
        err.unwrap_err(),
        crate::GitPurgeError::SafetyViolation(_)
    ));

    // 6. Restore again with force should succeed
    let spec_force = RestoreSpec {
        branch: BranchName("merged-branch".to_string()),
        as_tag: false,
        target_name: None,
        force: true,
        original_ref: None,
    };
    let outcome2 = engine.restore(&snapshot.id, spec_force).unwrap();
    assert_eq!(outcome2.created_ref, "refs/heads/merged-branch");

    // 7. Restore as tag
    let spec_tag = RestoreSpec {
        branch: BranchName("merged-branch".to_string()),
        as_tag: true,
        target_name: Some("restored-tag".to_string()),
        force: false,
        original_ref: None,
    };
    let outcome_tag = engine.restore(&snapshot.id, spec_tag).unwrap();
    assert_eq!(outcome_tag.created_ref, "refs/tags/restored-tag");
    assert_eq!(outcome_tag.tip.0, original_oid.to_string());
    assert!(source_repo.find_reference("refs/tags/restored-tag").is_ok());
}

#[test]
fn test_auto_restore_on_failure() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("test-failed-delete-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::new());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-failed-delete-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // 1. Scan and build plan to delete unmerged-branch
    let filter = ActionFilter {
        include_unmerged: true,
        specific_branches: vec![BranchName("unmerged-branch".to_string())],
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();

    // Verify the branch exists before we try to delete it
    let source_repo = git2::Repository::open(repo_fixture.path()).unwrap();
    assert!(source_repo
        .find_reference("refs/heads/unmerged-branch")
        .is_ok());

    // 2. Execute plan with simulated failure during deletion to trigger SAFE-05
    let mut is_restore_called = false;

    let run_res = crate::action::execute_deletions_with_guard(
        &engine.config.lock().unwrap(),
        engine.git.as_ref(),
        engine.history.as_ref(),
        engine.repos.lock().unwrap().get(&repo_id).unwrap(),
        &plan.actions,
        false,
        |_action| {
            let mut r = source_repo
                .find_reference("refs/heads/unmerged-branch")
                .unwrap();
            r.delete().unwrap();

            Err(crate::GitPurgeError::Git(
                "Simulated delete failure".to_string(),
            ))
        },
        |_, _| {
            is_restore_called = true;
            true // accept the restore
        },
    )
    .unwrap();

    assert_eq!(run_res.len(), 1);
    assert!(matches!(
        run_res[0],
        crate::model::ActionResult::Failed { .. }
    ));
    assert!(is_restore_called);

    // Verify that the branch was automatically restored!
    assert!(source_repo
        .find_reference("refs/heads/unmerged-branch")
        .is_ok());
}

#[test]
fn test_disk_size_sublinear() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("test-size-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::new());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-size-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // Create 5 snapshots of the repository without any changes
    let mut snapshots = Vec::new();
    for _ in 0..5 {
        let snap = engine
            .backup_create(&repo_id, BackupOptions::default())
            .unwrap();
        snapshots.push(snap);
    }

    // Verify we have 5 snapshots in history
    let listed = engine.history.list_snapshots(&repo_id).unwrap();
    assert_eq!(listed.len(), 5);

    // Verify the bare mirror directory exists and objects are shared
    let mirror_manager = crate::backup::BackupMirrorManager::new(&engine.config.lock().unwrap());
    let mirror_path = mirror_manager.resolve_mirror_path(&repo_id);
    assert!(mirror_path.exists());
}

#[test]
fn test_golden_reports() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("test-report-repo".to_string());

    let config = Config {
        backups_root: None,
        default_policy: Policy::default(),
        ..Default::default()
    };

    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::default());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-report-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::macros::datetime!(2026-07-01 12:00:00 UTC),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    let report = engine
        .report(
            &repo_id,
            crate::report::ReportType::Audit,
            crate::report::ReportFormat::Markdown,
        )
        .unwrap();
    insta::assert_snapshot!("audit_report_markdown", report.content);
}

#[test]
fn test_scan_cache_hit_and_invalidation() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("test-cache-repo".to_string());

    let mut config = Config::default();
    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::default());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config.clone(),
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-cache-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // 1. First scan (cache miss, populates cache)
    let res1 = engine.scan(&repo_id, ScanOptions::default()).unwrap();
    assert!(!res1.classifications.is_empty());

    // Cache must now contain 1 entry
    {
        let cache = engine.scan_cache.lock().unwrap();
        assert_eq!(cache.len(), 1);
        assert!(cache.contains_key(&repo_id));
    }

    // 2. Second scan (cache hit)
    let res2 = engine.scan(&repo_id, ScanOptions::default()).unwrap();
    assert_eq!(res1, res2);

    // 3. Change policy (cache invalidation because policy signature changes)
    config.default_policy.age = "30 days ago".to_string();
    engine.update_config(config);

    let _res3 = engine.scan(&repo_id, ScanOptions::default()).unwrap();
    assert_eq!(engine.scan_cache.lock().unwrap().len(), 1);

    // 4. Manual cache clear (belt and suspenders)
    engine.scan_cache.lock().unwrap().clear();
    assert_eq!(engine.scan_cache.lock().unwrap().len(), 0);
}

#[test]
fn test_backup_restore_original_ref() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("test-restore-orig-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(crate::git::CompositeGitBackend::new());
    let secrets = Box::new(crate::auth::FakeSecretStore::default());
    let history = Box::new(crate::history::FakeHistoryStore::default());
    let report_sink = Box::new(crate::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(crate::progress::NoopProgressSink);

    let engine = Engine::new(
        config,
        git_backend,
        secrets,
        history,
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "test-restore-orig-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: crate::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // 1. Create a snapshot with only_branches
    let snapshot = engine
        .backup_create(
            &repo_id,
            BackupOptions {
                trigger: Some(crate::model::SnapshotTrigger::Manual),
                verify: true,
                only_branches: vec![BranchName("merged-branch".to_string())],
            },
        )
        .unwrap();

    // Check that refs contains the branch
    assert!(!snapshot.refs.is_empty());
    let ref_entry = &snapshot.refs[0];
    assert_eq!(ref_entry.branch.0, "merged-branch");

    // 2. Restore using original_ref spec
    let spec = RestoreSpec {
        branch: BranchName("merged-branch".to_string()),
        as_tag: false,
        target_name: Some("restored-ref-by-orig".to_string()),
        force: true,
        original_ref: Some(ref_entry.original_full_ref.clone()),
    };

    let outcome = engine.restore(&snapshot.id, spec).unwrap();
    assert_eq!(outcome.branch.0, "merged-branch");
    assert_eq!(outcome.created_ref, "refs/heads/restored-ref-by-orig");
}
