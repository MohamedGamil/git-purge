//! Testkit — deterministic fixture-repo builders (docs/12 testing-strategy).
//!
//! Behind the `testkit` feature gate so it never ships in production binaries.
//! Provides named builders that create on-disk git repos in temp dirs with fixed
//! authors, dates, and commit structure — no network, no machine-specific state.
//!
//! ## Builders (added incrementally per phase)
//! - `merged_repo` — repo with known merged + unmerged branches (P0-T5)
//! - `stale_repo` — repo with stale branches (P1)
//! - `multi_remote_repo` — repo with multiple remotes (P1)
//! - `naming_repo` — branches with various naming patterns (P1)

// TODO(P0-T5): implement builders module with FixtureRepo struct and named builders.
// TODO(P0-T5): ensure determinism: fixed author/committer identity + timestamps via FakeClock.
