#![allow(deprecated)]
//! Markdown report generation (doc 10 §5)

use crate::model::{
    Repository, ScanResult, TrendHistory, Classification, NamingVerdict,
    MergeState, Activity, TrendComparison, MetricDelta,
};

/// Helper to format date-time to human string.
fn format_datetime(dt: time::OffsetDateTime) -> String {
    dt.format(&time::format_description::parse(
        "[year]-[month]-[day] [hour]:[minute] UTC"
    ).unwrap()).unwrap_or_else(|_| dt.to_string())
}

/// Generate a Markdown Audit report.
pub fn generate_audit_report(
    repo: &Repository,
    scan: &ScanResult,
    generated_at: time::OffsetDateTime,
) -> String {
    let mut md = String::new();

    // Title
    md.push_str(&format!("# Git Purge Audit Report — {}\n\n", repo.display_name));
    md.push_str(&format!("Generated at: {}\n\n", format_datetime(generated_at)));

    // 1. Repository Overall Stats
    md.push_str("## 📊 Repository Overall Stats\n\n");
    md.push_str("| Metric | Count | Percentage | Description |\n");
    md.push_str("| --- | --- | --- | --- |\n");

    let total = scan.total_branches;
    let active = scan.classifications.iter().filter(|c| matches!(c.activity, Activity::Active)).count();
    let stale = scan.classifications.iter().filter(|c| matches!(c.activity, Activity::Stale)).count();
    let merged = scan.classifications.iter().filter(|c| matches!(c.merge_state, MergeState::Merged)).count();
    let unmerged = scan.classifications.iter().filter(|c| matches!(c.merge_state, MergeState::Unmerged)).count();
    let non_standard = scan.classifications.iter().filter(|c| !matches!(c.naming, NamingVerdict::Standard | NamingVerdict::Exempt { .. })).count();

    let pct = |count: usize| {
        if total == 0 {
            "0.0%".to_string()
        } else {
            format!("{:.1}%", (count as f64) / (total as f64) * 100.0)
        }
    };

    md.push_str(&format!("| **Total Branches** | {} | 100.0% | All tracked branches in the mirror |\n", total));
    md.push_str(&format!("| **Active Branches** | {} | {} | Branches with commits in the active window |\n", active, pct(active)));
    md.push_str(&format!("| **Stale Branches** | {} | {} | Branches with last commit older than threshold |\n", stale, pct(stale)));
    md.push_str(&format!("| **Merged Branches** | {} | {} | Branches fully merged into the default branch |\n", merged, pct(merged)));
    md.push_str(&format!("| **Unmerged Branches** | {} | {} | Branches with unmerged work |\n", unmerged, pct(unmerged)));
    md.push_str(&format!("| **Non-Standard Naming** | {} | {} | Branches violating the naming policy |\n\n", non_standard, pct(non_standard)));

    // 2. Recommended for Deletion
    md.push_str("## 🗑️ Branches Recommended for Deletion\n\n");
    let recs: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| {
            matches!(
                c.recommendation,
                crate::model::Recommendation::DeleteMerged | crate::model::Recommendation::ArchiveStale
            )
        })
        .collect();

    if recs.is_empty() {
        md.push_str("No branches currently recommended for cleanup.\n\n");
    } else {
        md.push_str("| Branch Name | Last Commit Date | Author | Type | Reason |\n");
        md.push_str("| --- | --- | --- | --- | --- |\n");
        for r in &recs {
            let scope_str = match r.scope {
                crate::model::BranchScope::Local => "Local",
                crate::model::BranchScope::Remote => "Remote",
            };
            let reason = match r.recommendation {
                crate::model::Recommendation::DeleteMerged => "Fully merged",
                crate::model::Recommendation::ArchiveStale => "Stale branch",
                _ => "Unused",
            };
            let last_commit_date = r.tip.commit_date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap_or_default();
            md.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                r.branch.0, last_commit_date, r.tip.author.name, scope_str, reason
            ));
        }
        md.push_str("\n### 💻 Quick Commands\n");
        md.push_str("```bash\n");
        for r in &recs {
            md.push_str(&format!(
                "git-purge delete --repo {} --branch {} --execute\n",
                repo.id.0, r.branch.0
            ));
        }
        md.push_str("```\n\n");
    }

    // 3. Naming standard violations
    md.push_str("## ⚠️ Naming Standards Violations (Branching Strategy)\n\n");
    let violations: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.naming, NamingVerdict::NonStandard { .. }))
        .collect();

    if violations.is_empty() {
        md.push_str("No naming standards violations found.\n\n");
    } else {
        md.push_str("| Branch Name | Last Commit Date | Author | Issue Identified |\n");
        md.push_str("| --- | --- | --- | --- |\n");
        for v in &violations {
            let issue = match &v.naming {
                NamingVerdict::NonStandard { reason } => match reason {
                    crate::model::NamingViolation::NoCategoryPrefix => {
                        "Missing category prefix (should contain a '/' e.g. feature/name)".to_string()
                    }
                    crate::model::NamingViolation::WrongPrefixFormat { prefix } => {
                        format!("Wrong prefix casing or format for '{}'", prefix)
                    }
                    crate::model::NamingViolation::NonStandardPrefix { prefix } => {
                        format!("Non-standard prefix '{}' (should be feature/, fix/, refactor/, etc.)", prefix)
                    }
                    crate::model::NamingViolation::UnknownPrefix { prefix } => {
                        format!("Unknown/Non-standard prefix '{}'", prefix)
                    }
                },
                _ => "".to_string(),
            };
            let last_commit_date = v.tip.commit_date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap_or_default();
            md.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                v.branch.0, last_commit_date, v.tip.author.name, issue
            ));
        }
        md.push('\n');
    }

    // 4. Stale branches by age
    md.push_str("## 📅 Categorization of Stale Branches by Age\n\n");
    let stale_branches: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.activity, Activity::Stale))
        .collect();

    if stale_branches.is_empty() {
        md.push_str("No stale branches found.\n\n");
    } else {
        let mut group_1_2 = Vec::new();
        let mut group_2_3 = Vec::new();
        let mut group_3_plus = Vec::new();

        let one_year = std::time::Duration::from_secs(365 * 86400);
        let two_years = std::time::Duration::from_secs(2 * 365 * 86400);
        let three_years = std::time::Duration::from_secs(3 * 365 * 86400);

        for s in stale_branches {
            if s.age >= three_years {
                group_3_plus.push(s);
            } else if s.age >= two_years {
                group_2_3.push(s);
            } else if s.age >= one_year {
                group_1_2.push(s);
            }
        }

        let render_stale_group = |group: &[&Classification], title: &str, md_str: &mut String| {
            md_str.push_str(&format!("### {}\n\n", title));
            if group.is_empty() {
                md_str.push_str("No branches in this age group.\n\n");
            } else {
                md_str.push_str("| Branch Name | Last Commit Date | Author | Merged? | Type |\n");
                md_str.push_str("| --- | --- | --- | --- | --- |\n");
                for s in group {
                    let scope_str = match s.scope {
                        crate::model::BranchScope::Local => "Local",
                        crate::model::BranchScope::Remote => "Remote",
                    };
                    let merged_str = if s.merge_state == MergeState::Merged { "✅ Yes" } else { "❌ No" };
                    let last_commit_date = s.tip.commit_date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap_or_default();
                    md_str.push_str(&format!(
                        "| `{}` | {} | {} | {} | {} |\n",
                        s.branch.0, last_commit_date, s.tip.author.name, merged_str, scope_str
                    ));
                }
                md_str.push('\n');
            }
        };

        render_stale_group(&group_1_2, "1–2 Years Old", &mut md);
        render_stale_group(&group_2_3, "2–3 Years Old", &mut md);
        render_stale_group(&group_3_plus, "3+ Years Old", &mut md);
    }

    // 5. Stale Branches: Review Required (Unmerged)
    md.push_str("## 🔍 Stale Branches: Review Required (Unmerged)\n\n");
    let review_required: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.activity, Activity::Stale) && matches!(c.merge_state, MergeState::Unmerged))
        .collect();

    if review_required.is_empty() {
        md.push_str("No unmerged stale branches requiring review.\n\n");
    } else {
        md.push_str("| Branch Name | Last Commit Date | Author | Last Commit Message | Type |\n");
        md.push_str("| --- | --- | --- | --- | --- |\n");
        for r in &review_required {
            let scope_str = match r.scope {
                crate::model::BranchScope::Local => "Local",
                crate::model::BranchScope::Remote => "Remote",
            };
            let last_commit_date = r.tip.commit_date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap_or_default();
            let subject_trimmed = if r.tip.subject.len() > 60 {
                format!("{}...", &r.tip.subject[..57])
            } else {
                r.tip.subject.clone()
            };
            md.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                r.branch.0, last_commit_date, r.tip.author.name, subject_trimmed, scope_str
            ));
        }
        md.push('\n');
    }

    // 6. Categorization by Branch Prefix / Purpose
    md.push_str("## 🗂️ Categorization by Branch Prefix / Purpose\n\n");
    
    let mut features = Vec::new();
    let mut bugfixes = Vec::new();
    let mut hotfixes = Vec::new();
    let mut releases = Vec::new();
    let mut tickets = Vec::new();
    let mut others = Vec::new();

    // Simple ticket regex
    let ticket_re = regex::Regex::new(r"^[A-Z]+-\d+").unwrap();

    for c in &scan.classifications {
        let name = &c.branch.0;
        if name.starts_with("feature/") || name.starts_with("feat/") {
            features.push(c);
        } else if name.starts_with("bugfix/") || name.starts_with("bug/") || name.starts_with("fix/") {
            bugfixes.push(c);
        } else if name.starts_with("hotfix/") {
            hotfixes.push(c);
        } else if name.starts_with("release/") || name.starts_with("version/") {
            releases.push(c);
        } else if ticket_re.is_match(name) {
            tickets.push(c);
        } else {
            others.push(c);
        }
    }

    let render_prefix_group = |group: &[&Classification], title: &str, md_str: &mut String| {
        md_str.push_str(&format!("### {} ({} branches)\n\n", title, group.len()));
        if !group.is_empty() {
            md_str.push_str("| Branch Name | Last Commit Date | Author | Merged? | Type |\n");
            md_str.push_str("| --- | --- | --- | --- | --- |\n");
            for s in group {
                let scope_str = match s.scope {
                    crate::model::BranchScope::Local => "Local",
                    crate::model::BranchScope::Remote => "Remote",
                };
                let merged_str = if s.merge_state == MergeState::Merged { "✅ Yes" } else { "❌ No" };
                let last_commit_date = s.tip.commit_date.format(&time::format_description::parse("[year]-[month]-[day]").unwrap()).unwrap_or_default();
                md_str.push_str(&format!(
                    "| `{}` | {} | {} | {} | {} |\n",
                    s.branch.0, last_commit_date, s.tip.author.name, merged_str, scope_str
                ));
            }
            md_str.push('\n');
        }
    };

    render_prefix_group(&features, "Features", &mut md);
    render_prefix_group(&bugfixes, "Bug Fixes", &mut md);
    render_prefix_group(&hotfixes, "Hotfixes", &mut md);
    render_prefix_group(&releases, "Releases", &mut md);
    render_prefix_group(&tickets, "Ticket-Based", &mut md);
    render_prefix_group(&others, "Other/Uncategorized", &mut md);

    md
}

/// Generate Markdown Trend report.
pub fn generate_trend_report(
    repo: &Repository,
    history: &TrendHistory,
    generated_at: time::OffsetDateTime,
    baseline_id: Option<&str>,
) -> String {
    let mut md = String::new();

    md.push_str(&format!("# Git Purge Cleanup Progress & Trend Report — {}\n\n", repo.display_name));
    md.push_str(&format!("Generated at: {}\n\n", format_datetime(generated_at)));

    if history.entries.is_empty() {
        md.push_str("No recorded history entries found. Run scans or cleanups to seed the timeseries.\n");
        return md;
    }

    let current = history.entries.last().unwrap();

    // 1. Compare vs Previous
    md.push_str("### 🔄 Compare against Previous Run\n\n");
    if history.entries.len() < 2 {
        md.push_str("No previous run for trend delta comparison.\n\n");
    } else {
        let prev = &history.entries[history.entries.len() - 2];
        md.push_str(&format!(
            "Comparing current state to run on **{}**:\n\n",
            format_datetime(prev.recorded_at)
        ));
        let comparison = crate::history::trends::compare_entries(prev, current);
        render_comparison_table(&comparison, &mut md);
    }

    // 2. Compare vs Baseline
    md.push_str("### 📉 Compare against Baseline Run\n\n");
    let baseline_entry = if let Some(_id_str) = baseline_id {
        // Find matching entry
        // We can't query by ID easily from the entries unless we keep run IDs.
        // As a fallback, we use the earliest entry (runs[0])
        history.entries.first().unwrap()
    } else {
        history.entries.first().unwrap()
    };

    md.push_str(&format!(
        "Comparing current state to baseline run on **{}**:\n\n",
        format_datetime(baseline_entry.recorded_at)
    ));
    
    let baseline_comparison = crate::history::trends::compare_entries(baseline_entry, current);
    render_comparison_table(&baseline_comparison, &mut md);

    // Milestone Callout
    let stale_reduction = (baseline_entry.stale_count as i64) - (current.stale_count as i64);
    if stale_reduction > 0 {
        let pct_reduction = (stale_reduction as f64) / (baseline_entry.stale_count as f64) * 100.0;
        md.push_str(&format!(
            "> [!TIP]\n> **Cleanup Milestone**: Stale branches have been reduced by **{}** branches (**{:.1}%** reduction) since the baseline run!\n\n",
            stale_reduction, pct_reduction
        ));
    }

    // 3. Run History Log
    md.push_str("### 📜 Run History Log\n\n");
    md.push_str("| Run Date | Total | Active | Stale | Merged | Unmerged |\n");
    md.push_str("| :--- | :---: | :---: | :---: | :---: | :---: |\n");

    let mut reversed_entries = history.entries.clone();
    reversed_entries.reverse();

    for entry in &reversed_entries {
        let suffix = if entry.recorded_at == baseline_entry.recorded_at {
            " (Baseline)"
        } else {
            ""
        };
        md.push_str(&format!(
            "| {}{} | {} | {} | {} | {} | {} |\n",
            format_datetime(entry.recorded_at),
            suffix,
            entry.total_branches,
            entry.active_count,
            entry.stale_count,
            entry.merged_count,
            entry.unmerged_count
        ));
    }

    md
}

fn render_comparison_table(comparison: &TrendComparison, md: &mut String) {
    md.push_str("| Metric | Old Value | New Value | Absolute Change | Change Ratio (%) |\n");
    md.push_str("| :--- | :---: | :---: | :---: | :---: |\n");

    let format_row = |name: &str, delta: &MetricDelta, md_str: &mut String| {
        let abs_change_str = if delta.abs_change > 0 {
            format!("+{}", delta.abs_change)
        } else {
            delta.abs_change.to_string()
        };
        let ratio_change_str = format!("{:.1}%", delta.ratio_change);
        md_str.push_str(&format!(
            "| **{}** | {} | {} | **{}** | **{}** |\n",
            name, delta.old_value, delta.new_value, abs_change_str, ratio_change_str
        ));
    };

    format_row("Total Branches", &comparison.total, md);
    format_row("Stale Branches", &comparison.stale, md);
    format_row("Active Branches", &comparison.active, md);
    format_row("Merged Branches", &comparison.merged, md);
    format_row("Unmerged Branches", &comparison.unmerged, md);
    format_row("Non-Standard Naming", &comparison.non_standard, md);
    md.push('\n');
}
