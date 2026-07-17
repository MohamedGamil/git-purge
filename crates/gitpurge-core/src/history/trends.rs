//! Trend calculation algorithms (doc 10 §4.3)

use crate::model::{MergeState, MetricDelta, ScanResult, TrendComparison, TrendDiff, TrendEntry};
use std::collections::HashSet;
use time::OffsetDateTime;

/// Compare two trend entries (e.g., current run vs. previous or baseline run).
pub fn compare_entries(old: &TrendEntry, new: &TrendEntry) -> TrendComparison {
    TrendComparison {
        total: MetricDelta::calculate(old.total_branches, new.total_branches),
        active: MetricDelta::calculate(old.active_count, new.active_count),
        stale: MetricDelta::calculate(old.stale_count, new.stale_count),
        merged: MetricDelta::calculate(old.merged_count, new.merged_count),
        unmerged: MetricDelta::calculate(old.unmerged_count, new.unmerged_count),
        non_standard: MetricDelta::calculate(old.non_standard_count, new.non_standard_count),
    }
}

/// Compare two scan results over time, computing deltas, identifying newly
/// appeared/disappeared branches, and tracking merge velocity.
pub fn compare_scans(
    old_scan: &ScanResult,
    new_scan: &ScanResult,
    old_time: OffsetDateTime,
    new_time: OffsetDateTime,
) -> TrendDiff {
    let old_entry = TrendEntry::from_scan(old_scan, old_time, 0, 0);
    let new_entry = TrendEntry::from_scan(new_scan, new_time, 0, 0);
    let comparison = compare_entries(&old_entry, &new_entry);

    let old_names: HashSet<&str> = old_scan
        .classifications
        .iter()
        .map(|c| c.branch.0.as_str())
        .collect();
    let new_names: HashSet<&str> = new_scan
        .classifications
        .iter()
        .map(|c| c.branch.0.as_str())
        .collect();

    let mut added_branches: Vec<String> = new_names
        .difference(&old_names)
        .map(|s| s.to_string())
        .collect();
    added_branches.sort();

    let mut removed_branches: Vec<String> = old_names
        .difference(&new_names)
        .map(|s| s.to_string())
        .collect();
    removed_branches.sort();

    // Identify transitioned branches: was Unmerged in old_scan, is Merged in new_scan.
    let old_unmerged: HashSet<&str> = old_scan
        .classifications
        .iter()
        .filter(|c| c.merge_state == MergeState::Unmerged)
        .map(|c| c.branch.0.as_str())
        .collect();

    let new_merged: HashSet<&str> = new_scan
        .classifications
        .iter()
        .filter(|c| c.merge_state == MergeState::Merged)
        .map(|c| c.branch.0.as_str())
        .collect();

    let transitioned: HashSet<_> = old_unmerged.intersection(&new_merged).cloned().collect();
    let merge_velocity = transitioned.len();

    let duration_secs = (new_time - old_time).whole_seconds();
    let merges_per_day = if duration_secs > 0 {
        (merge_velocity as f64) / (duration_secs as f64) * 86400.0
    } else {
        0.0
    };

    TrendDiff {
        comparison,
        added_branches,
        removed_branches,
        merge_velocity,
        merges_per_day,
    }
}
