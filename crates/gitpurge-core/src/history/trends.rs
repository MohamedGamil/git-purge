//! Trend calculation algorithms (doc 10 §4.3)

use crate::model::{TrendEntry, TrendComparison, MetricDelta};

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
