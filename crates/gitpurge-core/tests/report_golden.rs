use gitpurge_core::model::{
    Activity, Branch, BranchName, BranchScope, Classification, Commit, GitUrl, MergeState,
    NamingVerdict, NamingViolation, Oid, Protection, ProtectionReason, Recommendation, RefBasis,
    RepoId, Repository, ScanResult, Signature, TrackingFacet, TrendEntry, TrendHistory,
};
use gitpurge_core::report::{html, json, markdown};
use time::OffsetDateTime;

#[test]
fn test_report_renderers_golden() {
    let repo_id = RepoId("test-repo".to_string());

    let author = Signature {
        name: "Alice Developer".to_string(),
        email: "alice@example.com".to_string(),
        when: OffsetDateTime::from_unix_timestamp(1784278800).unwrap(),
    };

    let commit1 = Commit {
        oid: Oid("1111111111111111111111111111111111111111".to_string()),
        short: "1111111".to_string(),
        author: author.clone(),
        committer: author.clone(),
        author_date: OffsetDateTime::from_unix_timestamp(1784278800).unwrap(),
        commit_date: OffsetDateTime::from_unix_timestamp(1784278800).unwrap(),
        subject: "Add login feature".to_string(),
        parents: Vec::new(),
    };

    let commit2 = Commit {
        oid: Oid("2222222222222222222222222222222222222222".to_string()),
        short: "2222222".to_string(),
        author: author.clone(),
        committer: author.clone(),
        author_date: OffsetDateTime::from_unix_timestamp(1784200000).unwrap(),
        commit_date: OffsetDateTime::from_unix_timestamp(1784200000).unwrap(),
        subject: "Wip login stub".to_string(),
        parents: Vec::new(),
    };

    let repo = Repository {
        id: repo_id.clone(),
        display_name: "Test Repository".to_string(),
        local_path: Some(std::path::PathBuf::from("/workspace/test-repo")),
        remote_url: GitUrl::parse("https://github.com/MohamedGamil/git-purge.git").ok(),
        default_branch: Some(Branch {
            name: BranchName("main".to_string()),
            scope: BranchScope::Local,
            remote: None,
            full_ref: "refs/heads/main".to_string(),
            tip: commit1.clone(),
            is_head: true,
            upstream: None,
        }),
        provider: gitpurge_core::model::ProviderHint::Unknown,
        added_at: OffsetDateTime::from_unix_timestamp(1784278800).unwrap(),
        last_scanned_at: None,
    };

    let class1 = Classification {
        branch: BranchName("feature/login".to_string()),
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
        tip: commit1.clone(),
        recommendation: Recommendation::DeleteMerged,
    };

    let class2 = Classification {
        branch: BranchName("wip-stub".to_string()),
        scope: BranchScope::Local,
        remote: None,
        upstream: None,
        merge_state: MergeState::Unmerged,
        activity: Activity::Stale,
        age: std::time::Duration::from_secs(365 * 24 * 3600 * 2), // 2 years old
        protection: Protection::Unprotected,
        naming: NamingVerdict::NonStandard {
            reason: NamingViolation::NoCategoryPrefix,
        },
        tracking: TrackingFacet {
            ahead: 2,
            behind: 0,
            upstream_gone: false,
            compared_against: RefBasis::DefaultBranch,
        },
        tip: commit2.clone(),
        recommendation: Recommendation::ReviewUnmerged,
    };

    let class3 = Classification {
        branch: BranchName("main".to_string()),
        scope: BranchScope::Local,
        remote: None,
        upstream: None,
        merge_state: MergeState::Merged,
        activity: Activity::Active,
        age: std::time::Duration::from_secs(0),
        protection: Protection::Protected {
            reason: ProtectionReason::DefaultBranch,
        },
        naming: NamingVerdict::Standard,
        tracking: TrackingFacet {
            ahead: 0,
            behind: 0,
            upstream_gone: false,
            compared_against: RefBasis::DefaultBranch,
        },
        tip: commit1.clone(),
        recommendation: Recommendation::KeepProtected,
    };

    let scan_result = ScanResult {
        repo: repo_id.clone(),
        classifications: vec![class1, class2, class3],
        total_branches: 3,
        excluded_count: 0,
    };

    let history_time_base = OffsetDateTime::from_unix_timestamp(1784200000).unwrap();
    let history_time_curr = OffsetDateTime::from_unix_timestamp(1784278800).unwrap();

    let entry_base = TrendEntry {
        recorded_at: history_time_base,
        total_branches: 10,
        merged_count: 2,
        unmerged_count: 8,
        stale_count: 5,
        active_count: 5,
        deleted_count: 0,
        archived_count: 0,
        non_standard_count: 3,
        protected_count: 1,
    };

    let entry_curr = TrendEntry {
        recorded_at: history_time_curr,
        total_branches: 3,
        merged_count: 2,
        unmerged_count: 1,
        stale_count: 1,
        active_count: 2,
        deleted_count: 7,
        archived_count: 0,
        non_standard_count: 1,
        protected_count: 1,
    };

    let history = TrendHistory {
        repo: repo_id.clone(),
        entries: vec![entry_base, entry_curr],
    };

    let gen_time = OffsetDateTime::from_unix_timestamp(1784278800).unwrap();

    // 1. Markdown Audit Report
    let md_audit = markdown::generate_audit_report(&repo, &scan_result, gen_time);
    insta::assert_snapshot!("md_audit_report", md_audit);

    // 2. Markdown Trend Report
    let md_trend = markdown::generate_trend_report(&repo, &history, gen_time, None);
    insta::assert_snapshot!("md_trend_report", md_trend);

    // 3. JSON Report
    let json_report =
        json::generate_json_report(&repo, &scan_result, Some(&history), gen_time).unwrap();
    insta::assert_snapshot!("json_report", json_report);

    // 4. HTML Report
    let html_report = html::generate_html_report(&repo, &scan_result, Some(&history), gen_time);
    insta::assert_snapshot!("html_report", html_report);
}
