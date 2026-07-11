//! Policy — user-configured rules (docs/03 §5).
//!
//! `Policy` is the *only* source of thresholds and rules; nothing is hardcoded (the
//! anti-goal from CONVENTIONS §5). It is loaded from `config.toml`, merged with
//! per-repo overrides and CLI/UI flags.
//!
//! Note on regexes: naming rules are stored here as **pattern source strings**
//! (`RegexSource`) so `Policy` stays `Serialize`/`Deserialize`. The policy engine
//! (`crate::policy`) compiles them to `regex::Regex` at load time and surfaces a
//! `Config` error on an invalid pattern. (docs/03 sketches `Vec<Regex>`; regexes are
//! not serializable, hence the string-source split — see agent report.)

use serde::{Deserialize, Serialize};

use super::{BranchName, NamingViolation};

/// A glob pattern source (e.g. `"release/*"`, `"*-do-not-delete"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobPattern(pub String);

/// A regular-expression source string, compiled by the policy engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegexSource(pub String);

/// A case-insensitive substring exception (e.g. `"sdf"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiSubstring(pub String);

/// Maps a branch prefix to the naming violation it should report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixRule {
    /// The prefix to match (e.g. `"bugfix"`).
    pub prefix: String,
    /// The violation to report for it.
    pub violation: NamingViolation,
}

/// Naming policy — generalizes the legacy hardcoded regex + special cases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NamingPolicy {
    /// Ordered allowed regex sources; first hit wins → `Standard`.
    pub allowed: Vec<RegexSource>,
    /// Literal names that are exempt.
    pub exact_exceptions: Vec<String>,
    /// Case-insensitive contains → exempt.
    pub substring_exceptions: Vec<CiSubstring>,
    /// Prefix → violation reason mapping.
    pub remediation_map: Vec<PrefixRule>,
}

/// Protection policy. `well_known` is seeded and immutable: the resolver unions it
/// with user config, so a user can *add* protected names/globs but can never *remove*
/// the six well-known ones (docs/03 §5, SAFE-02).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectionPolicy {
    /// Seeded, immutable defaults: main, master, develop, staging, production, HEAD.
    pub well_known: Vec<String>,
    /// User additions (e.g. "main-legacy").
    pub protected_names: Vec<String>,
    /// User glob patterns (e.g. "release/*").
    pub protected_globs: Vec<GlobPattern>,
}

impl Default for ProtectionPolicy {
    fn default() -> Self {
        // The immutable well-known set (CONVENTIONS §7.3). `config`/policy loading
        // unions user additions on top of this; it can never be shrunk.
        Self {
            well_known: ["main", "master", "develop", "staging", "production", "HEAD"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            protected_names: Vec::new(),
            protected_globs: Vec::new(),
        }
    }
}

/// Default-branch resolution policy: explicit config → remote HEAD → candidates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultBranchPolicy {
    /// Explicit override, if configured.
    pub explicit: Option<BranchName>,
    /// Ordered candidates; default `["main", "master", "develop"]`.
    pub candidates: Vec<BranchName>,
}

impl Default for DefaultBranchPolicy {
    fn default() -> Self {
        Self {
            explicit: None,
            candidates: ["main", "master", "develop"]
                .iter()
                .map(|s| BranchName((*s).to_string()))
                .collect(),
        }
    }
}

/// User-configured thresholds & rules. The single source of policy truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Policy {
    /// Age threshold source string (default `"1 year ago"`). Parsed into an
    /// `AgeThreshold` by the policy engine; kept as a string here for config.
    pub age: String,
    /// Naming policy.
    pub naming: NamingPolicy,
    /// Protection policy.
    pub protection: ProtectionPolicy,
    /// Branches to omit from scan/plan entirely.
    pub excludes: Vec<GlobPattern>,
    /// Default-branch resolution policy.
    pub default_branch: DefaultBranchPolicy,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            // Matches the legacy scripts' `--age "1 year ago"` default (CONVENTIONS §9).
            age: "1 year ago".to_string(),
            naming: NamingPolicy::default(),
            protection: ProtectionPolicy::default(),
            excludes: Vec::new(),
            default_branch: DefaultBranchPolicy::default(),
        }
    }
}

/// A partial policy overlay applied on top of `Policy` from CLI flags / UI controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PolicyPatch {
    /// Override the age threshold.
    pub age: Option<String>,
    /// Additional excludes to layer on.
    pub excludes: Vec<GlobPattern>,
    /// Additional protected globs to layer on.
    pub protected_globs: Vec<GlobPattern>,
}
