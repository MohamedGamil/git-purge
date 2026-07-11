//! Filtering and sorting for branch classification lists.

use crate::model::{
    Activity, Classification, MergeState, Protection, RefFilter, SortField, SortOrder,
};

/// Apply a `RefFilter` to retain and sort classifications in-place.
pub fn filter_and_sort_classifications(
    classifications: &mut Vec<Classification>,
    filter: &RefFilter,
) {
    // 1. Filtering
    classifications.retain(|c| {
        // Search filter (substring match on branch name)
        if let Some(ref search) = filter.search {
            if !c.branch.0.contains(search) {
                return false;
            }
        }

        // Merged filter
        if let Some(merged) = filter.merged {
            let is_merged = c.merge_state == MergeState::Merged;
            if is_merged != merged {
                return false;
            }
        }

        // Stale filter
        if let Some(stale) = filter.stale {
            let is_stale = c.activity == Activity::Stale;
            if is_stale != stale {
                return false;
            }
        }

        // Protected filter
        if let Some(protected) = filter.protected {
            let is_protected = !matches!(c.protection, Protection::Unprotected);
            if is_protected != protected {
                return false;
            }
        }

        // Scope filter
        if let Some(scope) = filter.scope {
            if c.scope != scope {
                return false;
            }
        }

        true
    });

    // 2. Sorting
    if let Some(field) = filter.sort_by {
        let order = filter.sort_order.unwrap_or(SortOrder::Ascending);
        classifications.sort_by(|a, b| {
            let cmp = match field {
                SortField::Name => a.branch.0.cmp(&b.branch.0),
                SortField::Date => a.tip.commit_date.cmp(&b.tip.commit_date),
                SortField::Age => a.age.cmp(&b.age),
                SortField::Author => a.tip.author.name.cmp(&b.tip.author.name),
                SortField::MergeState => {
                    let a_val = match a.merge_state {
                        MergeState::Merged => 0,
                        MergeState::Unmerged => 1,
                        MergeState::Unknown => 2,
                    };
                    let b_val = match b.merge_state {
                        MergeState::Merged => 0,
                        MergeState::Unmerged => 1,
                        MergeState::Unknown => 2,
                    };
                    a_val.cmp(&b_val)
                }
                SortField::Activity => {
                    let a_val = match a.activity {
                        Activity::Active => 0,
                        Activity::Stale => 1,
                    };
                    let b_val = match b.activity {
                        Activity::Active => 0,
                        Activity::Stale => 1,
                    };
                    a_val.cmp(&b_val)
                }
                SortField::Ahead => a.tracking.ahead.cmp(&b.tracking.ahead),
                SortField::Behind => a.tracking.behind.cmp(&b.tracking.behind),
            };

            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BranchName, BranchScope, Commit, Recommendation, RefBasis, Signature, TrackingFacet,
    };
    use time::OffsetDateTime;

    fn dummy_classification(name: &str, is_merged: bool, age_secs: u64) -> Classification {
        Classification {
            branch: BranchName(name.to_string()),
            scope: BranchScope::Local,
            merge_state: if is_merged {
                MergeState::Merged
            } else {
                MergeState::Unmerged
            },
            activity: if age_secs > 100 {
                Activity::Stale
            } else {
                Activity::Active
            },
            age: std::time::Duration::from_secs(age_secs),
            protection: Protection::Unprotected,
            naming: crate::model::NamingVerdict::Standard,
            tracking: TrackingFacet {
                ahead: 0,
                behind: 0,
                upstream_gone: false,
                compared_against: RefBasis::DefaultBranch,
            },
            tip: Commit {
                oid: crate::model::Oid("abc".to_string()),
                short: "abc".to_string(),
                author: Signature {
                    name: "Author".to_string(),
                    email: "a@example.com".to_string(),
                    when: OffsetDateTime::UNIX_EPOCH,
                },
                committer: Signature {
                    name: "Author".to_string(),
                    email: "a@example.com".to_string(),
                    when: OffsetDateTime::UNIX_EPOCH,
                },
                author_date: OffsetDateTime::UNIX_EPOCH,
                commit_date: OffsetDateTime::UNIX_EPOCH,
                subject: "Subject".to_string(),
                parents: Vec::new(),
            },
            recommendation: Recommendation::NoAction,
        }
    }

    #[test]
    fn test_filter_and_sort() {
        let mut list = vec![
            dummy_classification("z-branch", true, 200),
            dummy_classification("a-branch", false, 50),
        ];

        // Filter for merged only
        let filter = RefFilter {
            merged: Some(true),
            ..Default::default()
        };
        filter_and_sort_classifications(&mut list, &filter);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].branch.0, "z-branch");

        // Sort by name ascending
        let mut list2 = vec![
            dummy_classification("z-branch", true, 200),
            dummy_classification("a-branch", false, 50),
        ];
        let filter_sort = RefFilter {
            sort_by: Some(SortField::Name),
            sort_order: Some(SortOrder::Ascending),
            ..Default::default()
        };
        filter_and_sort_classifications(&mut list2, &filter_sort);
        assert_eq!(list2[0].branch.0, "a-branch");
        assert_eq!(list2[1].branch.0, "z-branch");
    }
}
