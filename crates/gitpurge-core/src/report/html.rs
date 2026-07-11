#![allow(deprecated)]
//! HTML report generation (doc 10 §5)

use crate::model::{
    Activity, Classification, MergeState, NamingVerdict, Repository, ScanResult, TrendHistory,
};

/// Helper to format date-time.
fn format_datetime(dt: time::OffsetDateTime) -> String {
    dt.format(&time::format_description::parse("[year]-[month]-[day] [hour]:[minute] UTC").unwrap())
        .unwrap_or_else(|_| dt.to_string())
}

/// Generate a self-contained HTML report with One Dark Pro styling.
pub fn generate_html_report(
    repo: &Repository,
    scan: &ScanResult,
    history: Option<&TrendHistory>,
    generated_at: time::OffsetDateTime,
) -> String {
    let mut html = String::new();

    // Stats calculations
    let total = scan.total_branches;
    let active = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.activity, Activity::Active))
        .count();
    let stale = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.activity, Activity::Stale))
        .count();
    let merged = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.merge_state, MergeState::Merged))
        .count();
    let unmerged = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.merge_state, MergeState::Unmerged))
        .count();
    let non_standard = scan
        .classifications
        .iter()
        .filter(|c| {
            !matches!(
                c.naming,
                NamingVerdict::Standard | NamingVerdict::Exempt { .. }
            )
        })
        .count();

    let pct = |count: usize| {
        if total == 0 {
            "0.0%".to_string()
        } else {
            format!("{:.1}%", (count as f64) / (total as f64) * 100.0)
        }
    };

    // Header & Styles
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Git Purge Audit Report</title>
    <style>
        :root {
            /* One Dark Pro theme variables */
            --bg-deep: #21252b;
            --bg: #282c34;
            --bg-raised: #2c313a;
            --fg: #abb2bf;
            --fg-strong: #d7dae0;
            --border: #3e4451;
            
            --red: #e06c75;
            --green: #98c379;
            --yellow: #e5c07b;
            --orange: #d19a66;
            --blue: #61afef;
            --purple: #c678dd;
            --cyan: #56b6c2;
            --gray: #5c6370;
            --accent: #61afef;
        }

        @media (prefers-color-scheme: light) {
            :root {
                /* One Light theme variables */
                --bg-deep: #f0f0f1;
                --bg: #fafafa;
                --bg-raised: #ffffff;
                --fg: #383a42;
                --fg-strong: #202227;
                --border: #d4d4d6;
                
                --red: #e45649;
                --green: #50a14f;
                --yellow: #c18401;
                --orange: #b76b01;
                --blue: #4078f2;
                --purple: #a626a4;
                --cyan: #0184bc;
                --gray: #a0a1a7;
                --accent: #4078f2;
            }
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: var(--bg-deep);
            color: var(--fg);
            margin: 0;
            padding: 24px;
            line-height: 1.5;
        }

        .container {
            max-width: 1000px;
            margin: 0 auto;
        }

        header {
            margin-bottom: 32px;
            padding-bottom: 16px;
            border-bottom: 1px solid var(--border);
        }

        h1, h2, h3, h4 {
            color: var(--fg-strong);
            margin-top: 0;
        }

        .subtitle {
            color: var(--gray);
            font-size: 14px;
        }

        .card {
            background-color: var(--bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 24px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }

        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 20px;
            margin-bottom: 24px;
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 20px;
            text-align: left;
        }

        th, td {
            padding: 12px;
            border-bottom: 1px solid var(--border);
        }

        th {
            background-color: var(--bg-raised);
            color: var(--fg-strong);
            font-weight: 600;
        }

        tr:hover {
            background-color: rgba(255, 255, 255, 0.02);
        }

        .badge {
            display: inline-block;
            padding: 2px 8px;
            font-size: 12px;
            font-weight: 600;
            border-radius: 4px;
            text-transform: uppercase;
        }

        .badge-success { background-color: rgba(152, 195, 121, 0.15); color: var(--green); }
        .badge-warning { background-color: rgba(229, 192, 123, 0.15); color: var(--yellow); }
        .badge-danger { background-color: rgba(224, 108, 117, 0.15); color: var(--red); }
        .badge-info { background-color: rgba(97, 175, 239, 0.15); color: var(--blue); }

        pre {
            background-color: var(--bg-deep);
            color: var(--cyan);
            padding: 16px;
            border-radius: 6px;
            overflow-x: auto;
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
            font-size: 14px;
            border: 1px solid var(--border);
        }

        .tip-callout {
            background-color: rgba(97, 175, 239, 0.1);
            border-left: 4px solid var(--blue);
            padding: 16px;
            border-radius: 0 6px 6px 0;
            margin-bottom: 20px;
        }

        .tip-callout p {
            margin: 0;
            color: var(--fg-strong);
        }

        .stale-bucket {
            margin-top: 16px;
            border-left: 2px solid var(--border);
            padding-left: 16px;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
"#);

    html.push_str(&format!(
        "<h1>Git Purge Report — {}</h1>",
        repo.display_name
    ));
    html.push_str(&format!(
        "<p class=\"subtitle\">Generated at {}</p>",
        format_datetime(generated_at)
    ));
    html.push_str("</header>");

    // 1. Overall Stats Card
    html.push_str(
        "<div class=\"card\">
            <h2>📊 Repository Overall Stats</h2>
            <table>
                <thead>
                    <tr>
                        <th>Metric</th>
                        <th>Count</th>
                        <th>Percentage</th>
                        <th>Description</th>
                    </tr>
                </thead>
                <tbody>",
    );

    let rows = [
        (
            "Total Branches",
            total,
            "100.0%",
            "All tracked branches in the mirror",
        ),
        (
            "Active Branches",
            active,
            &pct(active),
            "Branches with commits in the active window",
        ),
        (
            "Stale Branches",
            stale,
            &pct(stale),
            "Branches with last commit older than threshold",
        ),
        (
            "Merged Branches",
            merged,
            &pct(merged),
            "Branches fully merged into the default branch",
        ),
        (
            "Unmerged Branches",
            unmerged,
            &pct(unmerged),
            "Branches with unmerged work",
        ),
        (
            "Non-Standard Naming",
            non_standard,
            &pct(non_standard),
            "Branches violating the naming policy",
        ),
    ];

    for r in &rows {
        html.push_str(&format!(
            "<tr><td><strong>{}</strong></td><td>{}</td><td>{}</td><td>{}</td></tr>",
            r.0, r.1, r.2, r.3
        ));
    }

    html.push_str("</tbody></table></div>");

    // 2. Recommended for Deletion
    html.push_str("<div class=\"card\"><h2>🗑️ Recommended for Deletion</h2>");
    let recs: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| {
            matches!(
                c.recommendation,
                crate::model::Recommendation::DeleteMerged
                    | crate::model::Recommendation::ArchiveStale
            )
        })
        .collect();

    if recs.is_empty() {
        html.push_str("<p>No branches recommended for cleanup.</p>");
    } else {
        html.push_str(
            "<table><thead><tr>
                <th>Branch Name</th><th>Last Commit</th><th>Author</th><th>Type</th><th>Reason</th>
            </tr></thead><tbody>",
        );
        for r in &recs {
            let scope_str = match r.scope {
                crate::model::BranchScope::Local => "Local",
                crate::model::BranchScope::Remote => "Remote",
            };
            let reason = match r.recommendation {
                crate::model::Recommendation::DeleteMerged => "Merged",
                crate::model::Recommendation::ArchiveStale => "Stale",
                _ => "Review",
            };
            let last_commit_date = r
                .tip
                .commit_date
                .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap_or_default();
            html.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td><span class=\"badge badge-danger\">{}</span></td></tr>",
                r.branch.0, last_commit_date, r.tip.author.name, scope_str, reason
            ));
        }
        html.push_str("</tbody></table>");
        html.push_str("<h3>💻 Quick Commands</h3><pre>");
        for r in &recs {
            html.push_str(&format!(
                "git-purge delete --repo {} --branch {} --execute\n",
                repo.id.0, r.branch.0
            ));
        }
        html.push_str("</pre>");
    }
    html.push_str("</div>");

    // 3. Naming Standard Violations
    html.push_str("<div class=\"card\"><h2>⚠️ Naming Standards Violations</h2>");
    let violations: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.naming, NamingVerdict::NonStandard { .. }))
        .collect();

    if violations.is_empty() {
        html.push_str("<p>No naming standard violations found.</p>");
    } else {
        html.push_str(
            "<table><thead><tr>
                <th>Branch Name</th><th>Last Commit</th><th>Author</th><th>Issue</th>
            </tr></thead><tbody>",
        );
        for v in &violations {
            let issue = match &v.naming {
                NamingVerdict::NonStandard { reason } => match reason {
                    crate::model::NamingViolation::NoCategoryPrefix => {
                        "Missing category prefix".to_string()
                    }
                    crate::model::NamingViolation::WrongPrefixFormat { prefix } => {
                        format!("Wrong format for '{}'", prefix)
                    }
                    crate::model::NamingViolation::NonStandardPrefix { prefix } => {
                        format!("Non-standard prefix '{}'", prefix)
                    }
                    crate::model::NamingViolation::UnknownPrefix { prefix } => {
                        format!("Unknown prefix '{}'", prefix)
                    }
                },
                _ => "".to_string(),
            };
            let last_commit_date = v
                .tip
                .commit_date
                .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap_or_default();
            html.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td></tr>",
                v.branch.0, last_commit_date, v.tip.author.name, issue
            ));
        }
        html.push_str("</tbody></table>");
    }
    html.push_str("</div>");

    // 4. Stale Categorization by Age
    html.push_str("<div class=\"card\"><h2>📅 Categorization of Stale Branches by Age</h2>");
    let stale_branches: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| matches!(c.activity, Activity::Stale))
        .collect();

    if stale_branches.is_empty() {
        html.push_str("<p>No stale branches found.</p>");
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

        let render_html_stale_group =
            |group: &[&Classification], title: &str, html_str: &mut String| {
                html_str.push_str(&format!("<div class=\"stale-bucket\"><h3>{}</h3>", title));
                if group.is_empty() {
                    html_str.push_str("<p>No branches in this age group.</p>");
                } else {
                    html_str.push_str(
                    "<table><thead><tr>
                        <th>Branch Name</th><th>Last Commit</th><th>Author</th><th>Merged?</th><th>Type</th>
                    </tr></thead><tbody>"
                );
                    for s in group {
                        let scope_str = match s.scope {
                            crate::model::BranchScope::Local => "Local",
                            crate::model::BranchScope::Remote => "Remote",
                        };
                        let merged_badge = if s.merge_state == MergeState::Merged {
                            "<span class=\"badge badge-success\">Yes</span>"
                        } else {
                            "<span class=\"badge badge-warning\">No</span>"
                        };
                        let last_commit_date = s
                            .tip
                            .commit_date
                            .format(
                                &time::format_description::parse("[year]-[month]-[day]").unwrap(),
                            )
                            .unwrap_or_default();
                        html_str.push_str(&format!(
                        "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        s.branch.0, last_commit_date, s.tip.author.name, merged_badge, scope_str
                    ));
                    }
                    html_str.push_str("</tbody></table>");
                }
                html_str.push_str("</div>");
            };

        render_html_stale_group(&group_1_2, "1–2 Years Old", &mut html);
        render_html_stale_group(&group_2_3, "2–3 Years Old", &mut html);
        render_html_stale_group(&group_3_plus, "3+ Years Old", &mut html);
    }
    html.push_str("</div>");

    // 5. Stale Branches: Review Required (Unmerged)
    html.push_str("<div class=\"card\"><h2>🔍 Stale Branches: Review Required (Unmerged)</h2>");
    let review_required: Vec<&Classification> = scan
        .classifications
        .iter()
        .filter(|c| {
            matches!(c.activity, Activity::Stale) && matches!(c.merge_state, MergeState::Unmerged)
        })
        .collect();

    if review_required.is_empty() {
        html.push_str("<p>No unmerged stale branches requiring review.</p>");
    } else {
        html.push_str(
            "<table><thead><tr>
                <th>Branch Name</th><th>Last Commit</th><th>Author</th><th>Last Commit Message</th><th>Type</th>
            </tr></thead><tbody>"
        );
        for r in &review_required {
            let scope_str = match r.scope {
                crate::model::BranchScope::Local => "Local",
                crate::model::BranchScope::Remote => "Remote",
            };
            let last_commit_date = r
                .tip
                .commit_date
                .format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap_or_default();
            let subject_trimmed = if r.tip.subject.len() > 60 {
                format!("{}...", &r.tip.subject[..57])
            } else {
                r.tip.subject.clone()
            };
            html.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                r.branch.0, last_commit_date, r.tip.author.name, subject_trimmed, scope_str
            ));
        }
        html.push_str("</tbody></table>");
    }
    html.push_str("</div>");

    // 6. Trend comparisons if history exists
    if let Some(hist) = history {
        if hist.entries.len() >= 2 {
            let current = hist.entries.last().unwrap();
            let prev = &hist.entries[hist.entries.len() - 2];
            let comp = crate::history::trends::compare_entries(prev, current);

            html.push_str("<div class=\"card\"><h2>🔄 Compare against Previous Run</h2>");
            html.push_str(&format!(
                "<p class=\"subtitle\">Comparing to run on {}</p>",
                format_datetime(prev.recorded_at)
            ));

            html.push_str(
                "<table><thead><tr>
                    <th>Metric</th><th>Old Value</th><th>New Value</th><th>Absolute Change</th><th>Change Ratio</th>
                </tr></thead><tbody>"
            );

            let mut render_comp_row = |name: &str, delta: &crate::model::MetricDelta| {
                let badge = if delta.abs_change < 0 {
                    "badge-success"
                } else if delta.abs_change > 0 {
                    "badge-danger"
                } else {
                    "badge-info"
                };
                let abs_str = if delta.abs_change > 0 {
                    format!("+{}", delta.abs_change)
                } else {
                    delta.abs_change.to_string()
                };
                html.push_str(&format!(
                    "<tr><td><strong>{}</strong></td><td>{}</td><td>{}</td><td><span class=\"badge {}\">{}</span></td><td>{:.1}%</td></tr>",
                    name, delta.old_value, delta.new_value, badge, abs_str, delta.ratio_change
                ));
            };

            render_comp_row("Total Branches", &comp.total);
            render_comp_row("Stale Branches", &comp.stale);
            render_comp_row("Active Branches", &comp.active);
            render_comp_row("Merged Branches", &comp.merged);
            render_comp_row("Unmerged Branches", &comp.unmerged);
            render_comp_row("Non-Standard Naming", &comp.non_standard);

            html.push_str("</tbody></table></div>");
        }
    }

    html.push_str("</div></body></html>");

    html
}
