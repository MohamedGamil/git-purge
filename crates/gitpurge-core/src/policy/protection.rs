//! Protection policy evaluation.

use crate::model::{GlobPattern, Protection, ProtectionPolicy, ProtectionReason};

/// Helper to match standard shell glob pattern (* and ?).
pub fn matches_glob(pattern: &str, s: &str) -> bool {
    let mut regex_str = String::new();
    regex_str.push('^');
    for c in pattern.chars() {
        match c {
            '*' => regex_str.push_str(".*"),
            '?' => regex_str.push('.'),
            other => {
                regex_str.push_str(&regex::escape(&other.to_string()));
            }
        }
    }
    regex_str.push('$');
    if let Ok(re) = regex::Regex::new(&regex_str) {
        re.is_match(s)
    } else {
        false
    }
}

/// Evaluator for checking if a branch is protected.
pub struct ProtectionEvaluator {
    policy: ProtectionPolicy,
}

impl ProtectionEvaluator {
    /// Create a new `ProtectionEvaluator`.
    pub fn new(policy: ProtectionPolicy) -> Self {
        Self { policy }
    }

    /// Check if a branch is protected.
    pub fn evaluate(
        &self,
        branch_name: &str,
        is_default_branch: bool,
        is_head: bool,
    ) -> Protection {
        if is_head {
            return Protection::Protected {
                reason: ProtectionReason::IsHead,
            };
        }

        if is_default_branch {
            return Protection::Protected {
                reason: ProtectionReason::DefaultBranch,
            };
        }

        // Check well-known list
        if self.policy.well_known.iter().any(|w| w == branch_name) {
            return Protection::Protected {
                reason: ProtectionReason::WellKnown(branch_name.to_string()),
            };
        }

        // Check user-listed names
        if self.policy.protected_names.iter().any(|u| u == branch_name) {
            return Protection::Protected {
                reason: ProtectionReason::UserListed(branch_name.to_string()),
            };
        }

        // Check user-listed globs
        for GlobPattern(pat) in &self.policy.protected_globs {
            if matches_glob(pat, branch_name) {
                return Protection::Protected {
                    reason: ProtectionReason::GlobMatch(pat.clone()),
                };
            }
        }

        Protection::Unprotected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_glob() {
        assert!(matches_glob("release/*", "release/v1.0"));
        assert!(!matches_glob("release/*", "main"));
        assert!(matches_glob("*-delete", "do-not-delete"));
    }

    #[test]
    fn test_protection_evaluator() {
        let policy = ProtectionPolicy {
            well_known: vec!["main".to_string(), "master".to_string()],
            protected_names: vec!["custom-protected".to_string()],
            protected_globs: vec![GlobPattern("release/*".to_string())],
        };
        let evaluator = ProtectionEvaluator::new(policy);

        assert!(matches!(
            evaluator.evaluate("custom-protected", false, false),
            Protection::Protected {
                reason: ProtectionReason::UserListed(_)
            }
        ));
        assert!(matches!(
            evaluator.evaluate("release/v1", false, false),
            Protection::Protected {
                reason: ProtectionReason::GlobMatch(_)
            }
        ));
        assert!(matches!(
            evaluator.evaluate("main", false, false),
            Protection::Protected {
                reason: ProtectionReason::WellKnown(_)
            }
        ));
        assert!(matches!(
            evaluator.evaluate("other", true, false),
            Protection::Protected {
                reason: ProtectionReason::DefaultBranch
            }
        ));
        assert!(matches!(
            evaluator.evaluate("other", false, true),
            Protection::Protected {
                reason: ProtectionReason::IsHead
            }
        ));
    }

    #[test]
    fn test_protection_policy_default_well_known() {
        let policy = ProtectionPolicy::default();
        let expected = vec![
            "main",
            "master",
            "main-legacy",
            "develop",
            "staging",
            "prod",
            "production",
            "HEAD",
        ];
        for branch in expected {
            assert!(
                policy.well_known.contains(&branch.to_string()),
                "Default well-known list must contain {}",
                branch
            );
        }
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_matches_glob_no_panic(pattern in "\\PC*", s in "\\PC*") {
            let _ = matches_glob(&pattern, &s);
        }

        #[test]
        fn test_matches_glob_wildcard_invariant(s in "\\PC*") {
            assert!(matches_glob("*", &s));
        }

        #[test]
        fn test_matches_glob_exact_match(s in "[a-zA-Z0-9]+") {
            assert!(matches_glob(&s, &s));
        }

        #[test]
        fn test_protection_evaluator_no_panic(
            branch_name in "\\PC*",
            is_default_branch in proptest::bool::ANY,
            is_head in proptest::bool::ANY,
        ) {
            let policy = ProtectionPolicy::default();
            let evaluator = ProtectionEvaluator::new(policy);
            let _ = evaluator.evaluate(&branch_name, is_default_branch, is_head);
        }
    }
}
