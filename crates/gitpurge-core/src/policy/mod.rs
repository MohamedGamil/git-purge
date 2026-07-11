//! Policy engine — age parsing, naming regex, protection checking (docs/03 §5).

pub mod age;
pub mod naming;
pub mod protection;

pub use age::parse_age_threshold;
pub use naming::NamingEvaluator;
pub use protection::{matches_glob, ProtectionEvaluator};

use crate::model::Policy;
use std::time::Duration;

/// The unified policy evaluator compiling age, naming, and protection policies.
pub struct PolicyEngine {
    /// The source policy configuration.
    pub policy: Policy,
    /// Evaluator for naming conventions.
    pub naming: NamingEvaluator,
    /// Evaluator for branch protection.
    pub protection: ProtectionEvaluator,
    /// Pre-parsed age threshold duration.
    pub age_duration: Duration,
}

impl PolicyEngine {
    /// Compile the raw `Policy` config into a unified `PolicyEngine`.
    pub fn new(policy: Policy) -> Result<Self, String> {
        let naming = NamingEvaluator::new(&policy.naming)?;
        let protection = ProtectionEvaluator::new(policy.protection.clone());
        let age_duration = parse_age_threshold(&policy.age)?;

        Ok(Self {
            policy,
            naming,
            protection,
            age_duration,
        })
    }

    /// Check if a branch name is excluded from scanning entirely.
    pub fn is_excluded(&self, branch_name: &str) -> bool {
        for pat in &self.policy.excludes {
            if matches_glob(&pat.0, branch_name) {
                return true;
            }
        }
        false
    }
}
