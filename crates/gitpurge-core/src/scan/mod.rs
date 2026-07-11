//! Classification pipeline (docs/04 §2 — scan subsystem).
//!
//! The `Scanner` takes a repository, the policy, and a clock, and produces a
//! [`ScanResult`] containing [`Classification`]s for each branch. This is a
//! pure-read operation — no mutations.
//!
//! The pipeline:
//! 1. Enumerate refs via `GitBackend`.
//! 2. Resolve default branch via `DefaultBranchPolicy`.
//! 3. For each branch: compute merge state, age, protection, naming, tracking.
//! 4. Apply filters and produce the `ScanResult`.

// TODO(P1-T4): implement the classification pipeline.
