//! Named safety regression tests (SAFE-01–07) (DoD §6).
//!
//! One named test per safety invariant. These tests may never be removed.

use gitpurge_core::{
    clock::FakeClock,
    model::{ActionFilter, BackupOptions, BranchName, ExecMode, RepoId, Repository, RestoreSpec},
    testkit, Config, Engine, GitPurgeError,
};

#[test]
fn safe_01_dry_run_default() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("safe-01-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = Box::new(gitpurge_core::history::FakeHistoryStore::default());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
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

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "safe-01-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // 1. Scan and plan
    let filter = ActionFilter {
        merged_only: true,
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();
    assert!(
        !plan.actions.is_empty(),
        "Should have some actions to delete"
    );

    // 2. Execute with DryRun mode
    let report = engine.execute(&plan, ExecMode::DryRun, false).unwrap();
    assert_eq!(report.mode, ExecMode::DryRun);
    assert_eq!(report.success_count, 0);
    assert_eq!(report.failure_count, 0);
    assert_eq!(report.skipped_count, plan.actions.len());

    // 3. Verify that the branch still exists in the source repo (no mutation)
    let source_repo = git2::Repository::open(repo_fixture.path()).unwrap();
    assert!(source_repo
        .find_branch("merged-branch", git2::BranchType::Local)
        .is_ok());
}

#[test]
fn safe_02_protected_refs_never_deleted() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("safe-02-repo".to_string());

    // Configure policy with custom protected globs
    let mut policy = gitpurge_core::model::Policy::default();
    policy
        .protection
        .protected_globs
        .push(gitpurge_core::model::GlobPattern("merged-*".to_string()));

    let config = Config {
        default_policy: policy,
        ..Default::default()
    };
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = Box::new(gitpurge_core::history::FakeHistoryStore::default());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
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

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "safe-02-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // Plan with include_unmerged to get all possible actions
    let filter = ActionFilter {
        include_unmerged: true,
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();

    // Assert that protected references (main, and merged-branch matching the glob) are NOT in actions.
    for action in &plan.actions {
        assert_ne!(action.branch.0, "main");
        assert_ne!(action.branch.0, "merged-branch");
    }

    assert!(plan.skipped_count >= 2);
}

#[test]
fn safe_03_tags_never_deleted_by_branch_ops() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("safe-03-repo".to_string());

    // Create a tag with the same name as our merged-branch
    {
        let repo = git2::Repository::open(repo_fixture.path()).unwrap();
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.tag(
            "merged-branch",
            commit.as_object(),
            &repo.signature().unwrap(),
            "My Tag",
            false,
        )
        .unwrap();
    }

    let config = Config::default();
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = Box::new(gitpurge_core::history::FakeHistoryStore::default());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
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

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "safe-03-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // 1. Scan and plan branch deletions
    let filter = ActionFilter {
        merged_only: true,
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();

    // 2. Execute deletion
    let report = engine.execute(&plan, ExecMode::Execute, false).unwrap();
    assert_eq!(report.success_count, 1);

    // 3. Verify that the branch was deleted but the tag still exists!
    let repo = git2::Repository::open(repo_fixture.path()).unwrap();
    assert!(repo
        .find_branch("merged-branch", git2::BranchType::Local)
        .is_err());
    assert!(
        repo.find_reference("refs/tags/merged-branch").is_ok(),
        "Tag should NOT have been deleted"
    );
}

#[test]
fn safe_04_verified_pre_op_snapshot() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("safe-04-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = Box::new(gitpurge_core::history::FakeHistoryStore::new());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
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

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "safe-04-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model.clone()).unwrap();

    let filter = ActionFilter {
        merged_only: true,
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();

    // 1. Run execution which should create a verified pre-op snapshot
    let report = engine.execute(&plan, ExecMode::Execute, false).unwrap();
    assert!(
        report.snapshot.is_some(),
        "Snapshot should have been created"
    );
    let snapshot_id = report.snapshot.unwrap();

    // Verify snapshot exists and is verified
    let verify_res = engine.backup_verify(&repo_id, &snapshot_id).unwrap();
    assert!(verify_res.ok, "Backup snapshot verification should pass");

    // 2. Test that passing no_backup skips backup creation
    let repo_fixture2 = testkit::merged_repo();
    let repo_id2 = RepoId("safe-04-repo-no-backup".to_string());
    let repo_model2 = Repository {
        id: repo_id2.clone(),
        display_name: "safe-04-repo-no-backup".to_string(),
        local_path: Some(repo_fixture2.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model2).unwrap();
    let plan2 = engine.plan(&repo_id2, &filter).unwrap();
    let report2 = engine.execute(&plan2, ExecMode::Execute, true).unwrap();
    assert!(
        report2.snapshot.is_none(),
        "no_backup=true should skip snapshot creation"
    );
}

#[test]
fn safe_05_failed_delete_offers_restore() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("safe-05-repo".to_string());

    let config = Config::default();
    let git_backend = gitpurge_core::git::CompositeGitBackend::new();
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = gitpurge_core::history::FakeHistoryStore::new();
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
    let progress = Box::new(gitpurge_core::progress::NoopProgressSink);

    let engine = Engine::new(
        config.clone(),
        Box::new(gitpurge_core::git::CompositeGitBackend::new()),
        secrets,
        Box::new(gitpurge_core::history::FakeHistoryStore::new()),
        report_sink,
        clock,
        progress,
    );

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "safe-05-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model.clone()).unwrap();

    let filter = ActionFilter {
        include_unmerged: true,
        specific_branches: vec![BranchName("unmerged-branch".to_string())],
        ..Default::default()
    };
    let plan = engine.plan(&repo_id, &filter).unwrap();

    let source_repo = git2::Repository::open(repo_fixture.path()).unwrap();
    assert!(source_repo
        .find_reference("refs/heads/unmerged-branch")
        .is_ok());

    // 1. Simulate failure during deletion where restore is accepted
    let mut is_restore_called = false;
    let _run_res = gitpurge_core::action::execute_deletions_with_guard(
        &config,
        &git_backend,
        &history,
        &repo_model,
        &plan.actions,
        false,
        |_action| Err(GitPurgeError::Git("Simulated deletion error".to_string())),
        |_, _| {
            is_restore_called = true;
            true // accept restore
        },
    )
    .unwrap();

    assert!(is_restore_called);
    // The branch should be restored successfully
    assert!(source_repo
        .find_reference("refs/heads/unmerged-branch")
        .is_ok());

    // 2. Simulate failure during deletion where restore is declined
    // Delete the branch first so it's gone
    let mut r = source_repo
        .find_reference("refs/heads/unmerged-branch")
        .unwrap();
    r.delete().unwrap();

    let mut is_restore_called_declined = false;
    let _run_res2 = gitpurge_core::action::execute_deletions_with_guard(
        &config,
        &git_backend,
        &history,
        &repo_model,
        &plan.actions,
        false,
        |_action| Err(GitPurgeError::Git("Simulated deletion error 2".to_string())),
        |_, _| {
            is_restore_called_declined = true;
            false // decline restore
        },
    )
    .unwrap();

    assert!(is_restore_called_declined);
    // The branch should still be deleted (declined restore)
    assert!(source_repo
        .find_reference("refs/heads/unmerged-branch")
        .is_err());
}

#[test]
fn safe_06_no_force_overwrite_restore() {
    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("safe-06-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = Box::new(gitpurge_core::history::FakeHistoryStore::new());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
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

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "safe-06-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    let snapshot = engine
        .backup_create(&repo_id, BackupOptions::default())
        .unwrap();

    // The branch `merged-branch` already exists. Attempt to restore it without force.
    let spec = RestoreSpec {
        branch: BranchName("merged-branch".to_string()),
        as_tag: false,
        target_name: None,
        force: false,
        original_ref: None,
    };

    let err = engine.restore(&snapshot.id, spec);
    assert!(err.is_err());
    assert!(matches!(
        err.unwrap_err(),
        GitPurgeError::SafetyViolation(_)
    ));

    // Now restore with force = true
    let spec_force = RestoreSpec {
        branch: BranchName("merged-branch".to_string()),
        as_tag: false,
        target_name: None,
        force: true,
        original_ref: None,
    };
    let outcome = engine.restore(&snapshot.id, spec_force).unwrap();
    assert_eq!(outcome.created_ref, "refs/heads/merged-branch");
}

#[test]
fn safe_07_no_secrets_in_output() {
    let secret_token = "MY_SUPER_SECRET_PAT_TOKEN_998877";

    let repo_fixture = testkit::merged_repo();
    let repo_id = RepoId("safe-07-repo".to_string());

    let config = Config::default();
    let git_backend = Box::new(gitpurge_core::git::CompositeGitBackend::new());
    let secrets = Box::new(gitpurge_core::auth::FakeSecretStore::default());
    let history = Box::new(gitpurge_core::history::FakeHistoryStore::new());
    let report_sink = Box::new(gitpurge_core::report::FakeReportSink::default());
    let clock = Box::new(FakeClock::new(
        time::macros::datetime!(2026-07-05 12:00:00 UTC),
    ));
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

    let repo_model = Repository {
        id: repo_id.clone(),
        display_name: "safe-07-repo".to_string(),
        local_path: Some(repo_fixture.path().to_path_buf()),
        remote_url: None,
        default_branch: None,
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: time::OffsetDateTime::now_utc(),
        last_scanned_at: None,
    };
    engine.register_repo(repo_model).unwrap();

    // Store a dummy credential using the secret store
    engine
        .auth_store(
            &repo_id,
            "origin",
            gitpurge_core::auth::CredentialKind::HttpsToken,
            secret_token.as_bytes(),
        )
        .unwrap();

    // Generate snapshot and check the manifest file content on disk.
    let snapshot = engine
        .backup_create(&repo_id, BackupOptions::default())
        .unwrap();
    let manifest_content = std::fs::read_to_string(&snapshot.manifest_path).unwrap();

    // Assert the secret token is not present in the snapshot manifest!
    assert!(
        !manifest_content.contains(secret_token),
        "Secret leaked in snapshot manifest"
    );

    // Generate a report
    let report = engine
        .report(
            &repo_id,
            gitpurge_core::report::ReportType::Audit,
            gitpurge_core::report::ReportFormat::Markdown,
        )
        .unwrap();
    assert!(
        !report.content.contains(secret_token),
        "Secret leaked in audit report"
    );

    // Test a retrieve only returns Credential with label/kind, not the raw secret
    if let Some(cred) = engine.auth_retrieve(&repo_id, "origin").unwrap() {
        let debug_str = format!("{:?}", cred);
        assert!(
            !debug_str.contains(secret_token),
            "Secret leaked in Credential Debug"
        );
    }
}
