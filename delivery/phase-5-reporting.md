# Phase 5 — Reporting & history/trends

`Status: Approved` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p5--reporting--historytrends-8-ed), [CONVENTIONS §5/§13](CONVENTIONS.md), [architecture §3](../docs/02-architecture.md), [domain model §7](../docs/03-domain-model.md), [10-reporting-and-history.md](../docs/10-reporting-and-history.md)`

## Goal

Make reporting and historical branch tracking first-class (Requirement 7). Provide
the real `HistoryStore` SQLite adapter (bundled `rusqlite`) with a versioned schema,
record every `RunReport`, compute trend diffs versus the previous run and a baseline
(porting `track_branch_trends.py`), and generate audit + trend reports in Markdown,
JSON, and HTML through the `ReportSink` port. Reports aggregate/redact PII
(`Signature.email`) per policy — never leak it. This phase parallelizes with P3/P4
since it depends only on P1.

**Milestone:** M1 (partial, audit reports) → M3 (CLI 1.0 reporting).

**Dependencies:** **P1** merged (scan/classification → the data reports summarize).
Consumed by **P3** (`report`/`history` CLI verbs) and **P4** (Reports screen).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P5-T1** | SQLite `HistoryStore` adapter + schema | Implement the `HistoryStore` port with `rusqlite` (bundled SQLite) at `<data_dir>/git-purge/history.db` ([CONVENTIONS §5](CONVENTIONS.md)); versioned schema + migrations for repos, runs, run-items, snapshots, and per-run branch stats. Keyed by `RepoId`. | `crates/gitpurge-core/src/history/sqlite.rs`, `crates/gitpurge-core/src/history/schema.sql`, `crates/gitpurge-core/src/history/migrate.rs` | P1-T6 | no | 1.5 | Fresh DB migrates to current version; insert+read a run round-trips; opening an older schema migrates forward without data loss; `rusqlite` stays out of CLI/UI crates (**R6**). |
| **P5-T2** | Run recording | Persist a `RunReport` (mode, snapshot ref, per-item `ActionOutcome`, `RunMetrics`) plus a scan snapshot of branch stats for trend computation. Redact PII before storage. | `crates/gitpurge-core/src/history/record.rs` | P5-T1 | no | 1 | Executing a plan records one run with correct counts/metrics; stored rows contain no raw `Signature.email` (**SAFE-07**); **R7**. |
| **P5-T3** | Trend diffs (previous + baseline) | Compute trend deltas vs the previous run and vs a configurable baseline (total/stale/merged/unmerged/non-standard counts, deltas, direction). Ports `track_branch_trends.py` (JSON history → structured diffs). | `crates/gitpurge-core/src/history/trends.rs`, `crates/gitpurge-core/src/model/trend.rs` | P5-T2 | no | 1.5 | From a seeded sequence of recorded runs, trend output matches the legacy `track_branch_trends.py` deltas on the same inputs; **R7**. |
| **P5-T4** | Report generation (md / json / html) | `ReportSink` adapters rendering an audit report (classification tables, naming audit, stats) and a trend report in Markdown, JSON, and self-contained HTML. Reproduces the `generate_reports.py` tables. Exports to a user path. | `crates/gitpurge-core/src/report/{markdown.rs,json.rs,html.rs,mod.rs}` | P1-T6 | yes | 2 | The Markdown audit report reproduces the legacy `generate_reports.py` tables for a fixture; JSON validates against the report schema; HTML is self-contained (no external assets); **R7**. |
| **P5-T5** | Engine wiring + CLI `report`/`history` | Wire `Engine::report(fmt)` and `Engine::history(repo)`; back the P3 `report`/`history` CLI verbs (thin render only). Formats selectable via `--format md|json|html`. | `crates/gitpurge-core/src/lib.rs`, `crates/gitpurge-cli/src/cmd/{report.rs,history.rs}` | P5-T3, P5-T4 | no | 1 | `git-purge report --format html` writes a valid report; `git-purge history --repo <id>` prints the trend table; both go through `Engine` (no logic in CLI). |
| **P5-T6** | Golden trend/report regression | Golden tests asserting the trend report reproduces the legacy progress-report tables from recorded runs, across all three formats. | `crates/gitpurge-core/tests/report_golden.rs`, `crates/gitpurge-core/tests/snapshots/*` | P5-T3, P5-T4 | yes | 1 | `insta` golden test for md/json/html is stable across runs and matches the legacy progress tables (ROADMAP P5 exit); **R7/R8**. |

Total ≈ 8 ED.

## Exit criteria

- The trend report reproduces the legacy progress-report tables from recorded runs
  (ROADMAP P5 exit).
- History is stored in SQLite with a migratable schema; runs are recorded; reports emit
  in md/json/html and export.
- No PII (`Signature.email`) or secrets appear in stored history or any report.

### Requirements & safety invariants satisfied

- **R7** (reports + historical tracking integral): P5-T2, P5-T3, P5-T4, P5-T5.
- **R6** (SQLite behind a port; not in CLI/UI): P5-T1.
- **R8** (golden regression tests): P5-T6.
- **SAFE-07** (no secrets/PII in reports or history): P5-T2, P5-T4 (redaction), tested
  in P5-T6.

## Risks & open questions

- **Schema evolution** — settle the migration strategy (embedded numbered migrations
  vs. `user_version` pragma) early; it becomes a compatibility surface for shipped DBs.
- **Legacy-table fidelity** — reproducing `generate_reports.py` / `track_branch_trends.py`
  output "exactly" depends on column choices and rounding; capture the target tables as
  golden fixtures and treat any divergence as a spec question for
  [10-reporting-and-history.md](../docs/10-reporting-and-history.md).
- **PII redaction policy** — [domain model §10](../docs/03-domain-model.md) defers the
  email redaction rule to doc 10; implement aggregate/redact by default and make the
  mode configurable.
- **Baseline definition** — "baseline" (first run? tagged run? config-pinned?) needs a
  concrete rule; default to the earliest recorded run for the repo and allow override.
