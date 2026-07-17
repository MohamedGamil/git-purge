//! Naming policy evaluation.

use crate::model::{NamingPolicy, NamingVerdict, NamingViolation, PrefixRule};
use regex::Regex;

/// Compiles and evaluates branch names against `NamingPolicy`.
pub struct NamingEvaluator {
    allowed_regexes: Vec<Regex>,
    exact_exceptions: Vec<String>,
    substring_exceptions: Vec<String>,
    remediation_map: Vec<PrefixRule>,
}

impl NamingEvaluator {
    /// Compile the regular expressions and build the evaluator.
    pub fn new(policy: &NamingPolicy) -> Result<Self, String> {
        let mut allowed_regexes = Vec::new();
        let allowed = if policy.allowed.is_empty() {
            &NamingPolicy::default().allowed
        } else {
            &policy.allowed
        };
        for r in allowed {
            let re = Regex::new(&r.0).map_err(|e| format!("Invalid regex '{}': {}", r.0, e))?;
            allowed_regexes.push(re);
        }

        Ok(Self {
            allowed_regexes,
            exact_exceptions: policy.exact_exceptions.clone(),
            substring_exceptions: policy
                .substring_exceptions
                .iter()
                .map(|s| s.0.to_lowercase())
                .collect(),
            remediation_map: policy.remediation_map.clone(),
        })
    }

    /// Evaluate a branch name.
    pub fn evaluate(&self, branch_name: &str) -> NamingVerdict {
        // 1. Exact exception
        if self.exact_exceptions.iter().any(|e| e == branch_name) {
            return NamingVerdict::Exempt {
                rule: "exact_exception".to_string(),
            };
        }

        // 2. Substring exception (case-insensitive)
        let lower_name = branch_name.to_lowercase();
        for sub in &self.substring_exceptions {
            if lower_name.contains(sub) {
                return NamingVerdict::Exempt {
                    rule: format!("substring_exception:{}", sub),
                };
            }
        }

        // 3. Allowed regexes
        for re in &self.allowed_regexes {
            if re.is_match(branch_name) {
                return NamingVerdict::Standard;
            }
        }

        // 4. Violation analysis
        if !branch_name.contains('/') {
            NamingVerdict::NonStandard {
                reason: NamingViolation::NoCategoryPrefix,
            }
        } else {
            let parts: Vec<&str> = branch_name.splitn(2, '/').collect();
            let prefix = parts[0].to_string();

            if let Some(rule) = self.remediation_map.iter().find(|r| r.prefix == prefix) {
                NamingVerdict::NonStandard {
                    reason: rule.violation.clone(),
                }
            } else {
                NamingVerdict::NonStandard {
                    reason: NamingViolation::NonStandardPrefix { prefix },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CiSubstring, NamingViolation, PrefixRule, RegexSource};

    #[test]
    fn test_naming_evaluator() {
        let policy = NamingPolicy {
            allowed: vec![
                RegexSource("^feature/.*$".to_string()),
                RegexSource("^bugfix/.*$".to_string()),
            ],
            exact_exceptions: vec!["main-legacy".to_string()],
            substring_exceptions: vec![CiSubstring("vue3".to_string())],
            remediation_map: vec![PrefixRule {
                prefix: "ticket".to_string(),
                violation: NamingViolation::NonStandardPrefix {
                    prefix: "ticket".to_string(),
                },
            }],
        };

        let evaluator = NamingEvaluator::new(&policy).unwrap();

        assert_eq!(evaluator.evaluate("feature/login"), NamingVerdict::Standard);
        assert_eq!(evaluator.evaluate("bugfix/crash"), NamingVerdict::Standard);
        assert_eq!(
            evaluator.evaluate("main-legacy"),
            NamingVerdict::Exempt {
                rule: "exact_exception".to_string()
            }
        );
        assert_eq!(
            evaluator.evaluate("upgrade/vue3-core"),
            NamingVerdict::Exempt {
                rule: "substring_exception:vue3".to_string()
            }
        );
        assert_eq!(
            evaluator.evaluate("wip"),
            NamingVerdict::NonStandard {
                reason: NamingViolation::NoCategoryPrefix
            }
        );
        assert_eq!(
            evaluator.evaluate("ticket/123"),
            NamingVerdict::NonStandard {
                reason: NamingViolation::NonStandardPrefix {
                    prefix: "ticket".to_string()
                }
            }
        );
        assert_eq!(
            evaluator.evaluate("invalid/prefix"),
            NamingVerdict::NonStandard {
                reason: NamingViolation::NonStandardPrefix {
                    prefix: "invalid".to_string()
                }
            }
        );
    }

    #[test]
    fn test_naming_evaluator_empty_allowed_fallback() {
        let policy = NamingPolicy {
            allowed: Vec::new(), // empty
            exact_exceptions: Vec::new(),
            substring_exceptions: Vec::new(),
            remediation_map: Vec::new(),
        };

        let evaluator = NamingEvaluator::new(&policy).unwrap();

        // Should fall back to the default naming convention regex, which includes feature/* and main-legacy
        assert_eq!(evaluator.evaluate("feature/login"), NamingVerdict::Standard);
        assert_eq!(evaluator.evaluate("main-legacy"), NamingVerdict::Standard);
        assert_eq!(evaluator.evaluate("fix/bug"), NamingVerdict::Standard);
        assert_eq!(
            evaluator.evaluate("invalid/prefix"),
            NamingVerdict::NonStandard {
                reason: NamingViolation::NonStandardPrefix {
                    prefix: "invalid".to_string()
                }
            }
        );
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_naming_evaluator_no_panic(branch_name in "\\PC*") {
            let policy = NamingPolicy::default();
            if let Ok(evaluator) = NamingEvaluator::new(&policy) {
                let _ = evaluator.evaluate(&branch_name);
            }
        }

        #[test]
        fn test_naming_evaluator_custom_policy(
            branch_name in "\\PC*",
            regex_str in "feature/[a-z]+",
        ) {
            let policy = NamingPolicy {
                allowed: vec![crate::model::RegexSource(regex_str)],
                exact_exceptions: vec!["main-legacy".to_string()],
                substring_exceptions: vec![crate::model::CiSubstring("vue3".to_string())],
                remediation_map: Vec::new(),
            };
            if let Ok(evaluator) = NamingEvaluator::new(&policy) {
                let _ = evaluator.evaluate(&branch_name);
            }
        }
    }
}
