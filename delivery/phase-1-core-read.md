# Phase 1 — Core read engine (git + classify + diff)

`Status: Draft` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p1--core-read-engine-12-ed), [CONVENTIONS §4/§8](CONVENTIONS.md), [architecture §3](../docs/02-architecture.md), [domain model §2–§7](../docs/03-domain-model.md), [ADR-0002](../docs/adr/ADR-0002-git-engine-hybrid.md)`

## Goal

Build the read-only heart of `gitpurge-core`: the hybrid `GitBackend` (gix primary,
git2 fallback, shell diagnostic), ref/commit reading, merge-base/ancestry, and the
deterministic classification pipeline that ports and generalizes
`generate_reports.py`'s logic into a policy-driven engine. Add filtering/sorting,
`diff` between refs, and `show_tree`/file-at-commit. At the end of P1 the tool can
*understand* a repository — `scan`, `plan` (dry-run only, no mutation), `diff`, and
`show` return correct data on fixture repos — without ever writing to a repo.

**Milestone:** M1 — Read-only insight.

**Dependencies:** **P0** merged (workspace, ports, `testkit`, error/config).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P1-T1** | gix backend: open + ref enumeration | Implement `GixBackend` (the `GitBackend` port impl): open local/bare repo, enumerate `refs/heads/*`, `refs/remotes/<remote>/*`, `refs/tags/*` into `model::Ref`/`Branch`/`Tag` (names, kinds, targets). Populate `Branch.scope`/`remote`/`full_ref`/`is_head`. | `crates/gitpurge-core/src/git/gix_backend.rs`, `crates/gitpurge-core/src/model/{refs.rs,branch.rs,tag.rs}` | P0-T4 | no | 2 | On `multi_remote_repo` fixture, backend lists exactly the seeded local + remote branches + tags with correct `RefKind` and `is_head`; **R1/R4**. |
| **P1-T2** | Commit + signature metadata | Read `Commit` (oid, short, author/committer `Signature`, author_date, **commit_date**, subject, parents) from the object DB with no working-tree access. `commit_date` drives staleness ([domain model §2/§4](../docs/03-domain-model.md)). | `crates/gitpurge-core/src/git/gix_backend.rs`, `crates/gitpurge-core/src/model/commit.rs` | P1-T1 | no | 1 | Tip commit of a fixture branch has expected SHA, subject, and `commit_date` equal to the seeded timestamp. |
| **P1-T3** | Ancestry / merge-base / merge state | `merge-base --is-ancestor`-style check and ahead/behind counts; derive `MergeState` (`Merged` iff tip is ancestor of resolved default branch; unresolved → `Unknown` treated as **Unmerged** for safety). Resolve default branch via `DefaultBranchPolicy`. | `crates/gitpurge-core/src/git/gix_backend.rs`, `crates/gitpurge-core/src/scan/merge.rs`, `crates/gitpurge-core/src/policy/default_branch.rs` | P1-T2 | no | 1.5 | On `merged_repo`: merged branch → `Merged`, unmerged → `Unmerged`; with default branch unreadable, result is `Unknown` and never classified deletable. |
| **P1-T4** | git2 + shell composite backend | Add `Git2Backend` (fallback) and `ShellGitBackend` (diagnostic/parity only); a `CompositeGitBackend` that routes reads to gix and delegates gaps to git2. Push/auth stubs left for P6. Prove a **second backend swaps in without facade change** (R6). | `crates/gitpurge-core/src/git/{git2_backend.rs,shell_backend.rs,composite.rs}` | P1-T1 | yes | 1 | Same ref-enumeration test passes when `Engine` is wired with `Git2Backend` instead of `GixBackend`; parity test compares gix vs shell output on a fixture — satisfies **R6** swap proof. |
| **P1-T5** | Policy engine: age + naming + protection | Parse `AgeThreshold` ("1 year ago"/"6 months"/"90d"); implement the naming classifier porting `is_standard_branch` (ordered: default/well-known → exact exception → substring exception → allowed-regex → violation analysis) producing `NamingVerdict`/`NamingViolation`; protection resolver unioning immutable `well_known` with user names/globs. All from [domain model §3.1/§5](../docs/03-domain-model.md). | `crates/gitpurge-core/src/policy/{mod.rs,age.rs,naming.rs,protection.rs}` | P0-T3 | yes | 2 | Golden test reproduces `generate_reports.py` verdicts for a table of names (`feature/x`→Standard, `bugfix/y`→NonStandardPrefix, `upgrade/vue3`→Exempt, `main`→Protected); **SAFE-02**: user config can add but never remove `well_known`. |
| **P1-T6** | Classification pipeline (`scan`) | Pure fn `Branch + Policy + Clock → Classification` (merge_state, activity, age, protection, naming, tracking, recommendation); assemble `ScanResult` + `ScanStats`. `now()` comes from injected `Clock` only. | `crates/gitpurge-core/src/scan/mod.rs`, `crates/gitpurge-core/src/model/classification.rs` | P1-T3, P1-T5 | no | 1.5 | Golden `insta` snapshot of `ScanResult` on `stale_repo` at a fixed `Clock`; re-running is byte-stable; recommendations are advisory only. |
| **P1-T7** | Filter + sort | `ActionFilter` predicates and `SortKey` ordering over classifications (scope/merged/include_unmerged/older_than/name globs/excludes/only_non_standard; sort asc/desc). Protection **always** excluded from destructive sets even when it appears in listings. | `crates/gitpurge-core/src/scan/filter.rs` | P1-T6 | yes | 0.5 | Filter `--merged --older-than "1 year ago" --exclude "release/*"` on a fixture returns exactly the expected set; a Protected branch appears in a listing but never in a `Plan`; **R3**. |
| **P1-T8** | Diff + show-tree / file-at-commit | `diff(a,b)` between two `RefSpec`s (changed files + stats) and `show_tree(at, path)` returning a `TreeView`/blob at an arbitrary ref/commit (no checkout). | `crates/gitpurge-core/src/diff/{mod.rs,tree.rs}`, `crates/gitpurge-core/src/model/diff.rs` | P1-T2 | yes | 1.5 | `diff` between two fixture branches matches a golden changed-file list; `show_tree` returns the correct blob bytes at an arbitrary historical SHA — **R1** (view-at-commit). |
| **P1-T9** | Engine wiring: scan / plan(dry-run) / diff / show | Wire `Engine::scan`, `Engine::plan` (**dry-run only** — builds a `Plan` of what delete/archive *would* do; `requires_snapshot` computed; no mutation), `Engine::diff`, `Engine::show_tree`. CLI/UI use these unchanged in later phases. | `crates/gitpurge-core/src/lib.rs`, `crates/gitpurge-core/src/scan/plan.rs` | P1-T6, P1-T7, P1-T8 | no | 1 | End-to-end fixture test: `scan`→`plan` yields a `Plan` whose items match filters and whose `skipped` lists protected refs with reasons; `plan` writes nothing (**SAFE-01** preview path). |

Total ≈ 12 ED.

## Exit criteria

- `scan`/`plan`/`diff` return correct data on fixture repos; classification matches
  golden snapshots (ROADMAP P1 exit).
- Merged/ancestry, age, naming, and protection facets all computed deterministically
  from `Policy` + injected `Clock` — no hardcoded thresholds or names.
- A second `GitBackend` (git2) swaps into `Engine` with no facade change (R6 proof).
- `show_tree`/file-at-commit returns the correct blob at an arbitrary SHA.

### Requirements & safety invariants satisfied

- **R1** (view-at-commit, local+remote): P1-T1, P1-T8.
- **R3** (explore/filter/sort/compare/diff): P1-T7, P1-T8.
- **R4** (track local+remote, view-at-commit): P1-T1, P1-T3.
- **R6** (shared abstractions, backend swap): P1-T4.
- **SAFE-01** (dry-run default): P1-T9 — `plan` is preview-only, touches nothing.
- **SAFE-02** (protected refs never actioned): P1-T5 (immutable `well_known`), P1-T7
  (structural exclusion from `Plan`).
- **SAFE-03** partially staged: tags are enumerated but never enter an Action set;
  the deletion-time guard is completed in P2/P3.

## Risks & open questions

- **gix maturity for merge-base/ancestry and diff** — if a read op is missing or
  awkward in gix, route it through `Git2Backend` in the composite (ADR-0002); keep the
  gix-first default. Track which ops fall back.
- **Ahead/behind basis with no upstream** — [domain model §10](../docs/03-domain-model.md)
  flags this: default to comparing against the default branch and record
  `TrackingFacet.compared_against`; confirm this is desired for remote-only branches.
- **Fetch/prune before scan** — `ScanOptions.fetch` defaults true, but P1 is read-only
  and network-free in tests; wire the flag but exercise fetch only in P6 with auth.
- **`Signature.email` is PII** — ensure it never enters logs or snapshots here; report
  redaction policy is P5's concern but the rule is global (SAFE-07).
