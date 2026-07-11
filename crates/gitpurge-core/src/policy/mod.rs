//! Policy engine — age parsing, naming regex, protection checking (docs/03 §5).
//!
//! The policy module compiles the user-configured [`Policy`](crate::model::Policy) into
//! a runtime evaluator. It answers questions like "is this branch stale?", "is this
//! branch protected?", "does this branch follow naming conventions?".
//!
//! This module is the bridge between the raw `Policy` (serde-friendly, stored as config)
//! and the computed [`Classification`](crate::model::Classification) (runtime facets).

// TODO(P1-T3): implement age threshold parsing (e.g. "1 year ago" → Duration).
// TODO(P1-T3): implement naming regex compilation and evaluation.
// TODO(P1-T3): implement protection checking (well-known + user-listed + globs).
