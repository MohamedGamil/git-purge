# P1 — Task Cards

> Phase: **1 — Core read engine** · Status: **Not started** · Est: 12 ED
> Depends on: P0

---

## P1-T1 · gix read adapter

**Goal:** Implement the gix (gitoxide) adapter for `GitBackend` — ref enumeration,
commit/date/author metadata, object lookup, tree/blob reads. Pure-Rust, no C deps.

**Files:** `crates/gitpurge-core/src/git/gix_backend.rs`

**Depends on:** P0-T4

**Acceptance:** Adapter enumerates local+remote branches on `merged_repo` and
`multi_remote_repo` fixtures; commit metadata matches seeded values.

---

## P1-T2 · git2 write adapter

**Goal:** Implement the git2 (libgit2) adapter for write operations: push delete, remote
interaction, credential callbacks. Fallback for paths gix doesn't yet cover.

**Files:** `crates/gitpurge-core/src/git/git2_backend.rs`

**Depends on:** P0-T4

**Acceptance:** Adapter can delete a branch on a local bare remote via push refspec;
credential callback exercises SSH key and HTTPS token paths.

---

## P1-T3 · Composite GitBackend (hybrid routing)

**Goal:** Wire gix + git2 into a composite `GitBackend` that routes each method to the
appropriate implementation per ADR-0002 capability table.

**Files:** `crates/gitpurge-core/src/git/composite.rs`, `crates/gitpurge-core/src/git/mod.rs`

**Depends on:** P1-T1, P1-T2

**Acceptance:** Composite uses gix for reads, git2 for pushes; test swaps the second
backend and proves the trait surface stays stable (R6 proof).

---

## P1-T4 · Classification pipeline

**Goal:** Port and generalize `generate_reports.py`'s classification logic into a
policy-driven engine. Compute facets: merged/unmerged, stale/active (age), protected,
naming convention, ahead/behind.

**Files:** `crates/gitpurge-core/src/scan/mod.rs`, `crates/gitpurge-core/src/scan/classifier.rs`

**Depends on:** P1-T3, P0-T3

**Acceptance:** `Scanner::classify` on `merged_repo` fixture returns correct Classification
for each branch; `SAFE-02` (protected branches never classified as deletable) proven.

---

## P1-T5 · Policy engine

**Goal:** `PolicyEngine` evaluates classification results against the loaded policy config.
Resolves protection (well-known + user globs), staleness threshold, naming patterns.

**Files:** `crates/gitpurge-core/src/policy/mod.rs`, `crates/gitpurge-core/src/policy/evaluator.rs`

**Depends on:** P1-T4, P0-T3

**Acceptance:** Policy with custom globs protects matching branches; `SAFE-02` and `SAFE-03`
hold under all configurations.

---

## P1-T6 · Filter, sort, diff, show

**Goal:** `RefFilter`/`SortOrder` applied to classified branch lists; `diff(a, b)` returns
file-level diff stats; `show_tree`/file-at-commit returns blob content.

**Files:** `crates/gitpurge-core/src/diff/mod.rs`, `crates/gitpurge-core/src/scan/filter.rs`

**Depends on:** P1-T3

**Acceptance:** Filter by age/merged/name produces correct subsets; diff output matches
golden; `show` returns correct blob at arbitrary SHA.

---

## P1-T7 · Engine wiring + scan/plan tests

**Goal:** Wire `Engine::scan`, `Engine::plan` (dry-run only), `Engine::diff`, `Engine::show_tree`.
Comprehensive test suite on fixture repos.

**Files:** `crates/gitpurge-core/src/lib.rs`, `crates/gitpurge-core/tests/`

**Depends on:** P1-T4, P1-T5, P1-T6

**Acceptance:** `scan` + `plan` on all fixtures produce correct, reproducible results;
`plan` in `DryRun` mode never mutates; `SAFE-01` (dry-run default) proven.
