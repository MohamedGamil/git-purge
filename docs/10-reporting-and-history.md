# 10 — Reporting & History

`Status: Approved` · `Owner: Reporting` · `Last-updated: 2026-07-11` ·
`Related: [../delivery/CONVENTIONS.md](../delivery/CONVENTIONS.md), [02-architecture.md](02-architecture.md), [04-core-spec.md](04-core-spec.md), [05-cli-spec.md](05-cli-spec.md), [06-ui-spec.md](06-ui-spec.md), [07-ui-design-system.md](07-ui-design-system.md), [08-backup-and-restore.md](08-backup-and-restore.md), [00-vision-and-scope.md](00-vision-and-scope.md)`

> Normative for the `report` module (`gitpurge-core::report`) and the `history`
> module (`gitpurge-core::history`). Both the CLI (`git-purge report`/`history`,
> [doc 05](05-cli-spec.md)) and the UI Reports screen ([doc 06](06-ui-spec.md))
> are thin faces over these. This satisfies **Requirement R7** ("Reports generation
> and historical branch tracking are integral").

---

## 1. Purpose & scope

Git Purge productizes two legacy scripts:

- `generate_reports.py` → the **branch audit report** (classify, naming audit,
  categorization by age/prefix/purpose, recommended deletions).
- `track_branch_trends.py` → the **trend / progress report** (diffs vs previous run
  and vs baseline, ratios, run-history log, milestone callouts).

Both were flat-file scripts hardcoded to two repos, backed by a flat
`branch_history_db.json`. Git Purge replaces the JSON with an embedded **SQLite**
store (`rusqlite`, bundled — CONVENTIONS §5) that preserves the exact legacy metric
set while adding rich per-branch history and multi-repo, multi-run querying.

The report module is a **port** (`ReportSink`, architecture §3): report *content* is
computed by the core, and *rendering* is pluggable per format (`md`/`json`/`html`).

## 2. Report types

### 2.1 Branch audit report (ports `generate_reports.py`)

Computed from a `scan`'s `ScanResult` (classification of every ref). Sections, in
order, reproducing the legacy layout:

1. **Repository Overall Stats** — a table with, per metric, `Count`, `Percentage`
   (of total), and a `Description`. Metrics: **Total Branches**, **Active Branches**
   (`< age threshold`), **Stale Branches** (`>= age threshold`), **Merged Branches**
   (safe to delete), **Unmerged Branches**, **Non-Standard Naming**. These six are
   the canonical metric set (§3.3) — identical names/semantics to the legacy DB.
2. **Branches Recommended for Deletion** — merged remote branches, then local
   branches to delete (merged or stale), each as a table plus a copy-paste
   "quick commands" block (in Git Purge these are `git-purge delete …` invocations,
   not raw `git push --delete`).
3. **Naming Standards Violations** — every non-standard branch with the detected
   **issue** (missing prefix, wrong-case prefix, non-standard prefix, unknown
   prefix). Policy regex (from the tech-debt branching strategy, ported verbatim):
   `^(main|develop|staging|production|release/\d+\.\d+\.\d+|feature/.+|fix/.+|refactor/.+|hotfix/.+)$`
   with special allowed exceptions `upgrade/vue3` and any branch containing `SDF`/`sdf`.
   The regex and exceptions are **policy** (CONVENTIONS §8), user-configurable in
   `config.toml`; the legacy values are the defaults.
4. **Categorization of Stale Branches by Age** — buckets (default `1–2y`, `2–3y`,
   `3y+`, derived from the `--age` threshold) each listing branch, date, author,
   merged?, type (local/remote).
5. **Stale Branches: Review Required (Unmerged)** — unmerged stale branches with last
   commit message; these are the `archive`/careful-delete candidates.
6. **Categorization by Branch Prefix / Purpose** — Features (`feature/`,`feat/`),
   Bug Fixes (`bugfix/`,`bug/`,`fix/`), Hotfixes, Releases (`release/`,`version/`),
   Ticket-Based (`[A-Z]+-\d+`), Other/Uncategorized.

### 2.2 Trend / progress report (ports `track_branch_trends.py`)

Computed from recorded **Runs** in the history store. Per repo:

- **Compare against Previous Run** — a comparison table
  `Metric | Old | New | Absolute Change | Change Ratio (%)`, for all six metrics.
  Ratio = `(new − old) / old × 100`, `0.0%` when old is `0` (matching legacy).
- **Compare against Baseline** — same table vs the earliest recorded run (or a
  chosen `--baseline <run-id>`), showing overall progress.
- **Cleanup Milestone callout** — a `> [!TIP]` note when stale branches dropped
  vs baseline (absolute + ratio reduction).
- **Run History Log** — newest-first table `Run Date | Total | Active | Stale |
  Merged | Unmerged`, with the earliest run tagged `(Baseline)`.

## 3. History store (SQLite)

### 3.1 Location & engine
`<data_dir>/git-purge/history.db`, opened via `rusqlite` with the **bundled**
SQLite (no system dependency), WAL mode. Path resolved by `directories`
(CONVENTIONS §5); overridable with `--data-dir`/config. Accessed only through the
`HistoryStore` port (architecture §3) — the CLI/UI never touch `rusqlite` directly.

### 3.2 Schema (DDL)

```sql
PRAGMA user_version = 1;            -- schema migrations key off this
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- One row per tracked repository (keyed by canonical URL + local-path hash, §5).
CREATE TABLE repos (
    id             TEXT PRIMARY KEY,          -- canonical repo-id (e.g. "backend")
    canonical_url  TEXT,                       -- normalized remote URL (nullable: local-only)
    local_path     TEXT,                       -- working dir (nullable: remote-only)
    path_hash      TEXT NOT NULL,              -- hash(canonical_url + local_path)
    display_name   TEXT NOT NULL,
    default_branch TEXT NOT NULL DEFAULT 'origin/main',
    created_at     TEXT NOT NULL,              -- ISO-8601 UTC
    tombstoned_at  TEXT                         -- set when `repo remove` untracks it
);

-- One row per recorded execution (scan / delete / archive / restore / report).
CREATE TABLE runs (
    id            TEXT PRIMARY KEY,            -- ULID/uuid
    repo_id       TEXT NOT NULL REFERENCES repos(id),
    command       TEXT NOT NULL,               -- 'scan'|'delete'|'archive'|'restore'|'report'
    mode          TEXT NOT NULL,               -- 'dry-run'|'execute'
    started_at    TEXT NOT NULL,               -- ISO-8601 UTC
    finished_at   TEXT,
    snapshot_id   TEXT,                         -- pre-op Snapshot, if any (FK to snapshots)
    age_threshold TEXT,                         -- e.g. '1 year ago' (policy in effect)
    actor         TEXT,                         -- OS user / UI
    tool_version  TEXT NOT NULL,                -- git-purge version
    exit_code     INTEGER,                      -- CLI exit code (§05.7), NULL for UI
    note          TEXT,
    UNIQUE (repo_id, started_at, command)
);

-- Repo-level metrics timeseries (one row per run). Supersedes branch_history_db.json.
-- The first six columns are the EXACT legacy metric set.
CREATE TABLE metrics (
    run_id        TEXT PRIMARY KEY REFERENCES runs(id) ON DELETE CASCADE,
    repo_id       TEXT NOT NULL REFERENCES repos(id),
    captured_at   TEXT NOT NULL,
    total         INTEGER NOT NULL,
    active        INTEGER NOT NULL,
    stale         INTEGER NOT NULL,
    merged        INTEGER NOT NULL,
    unmerged      INTEGER NOT NULL,
    non_standard  INTEGER NOT NULL,
    -- richer additions (NULL-safe for imported legacy rows):
    local_count   INTEGER,
    remote_count  INTEGER,
    protected     INTEGER,
    deleted       INTEGER,                      -- refs deleted in this run
    archived      INTEGER,                      -- refs archived in this run
    restored      INTEGER,                      -- refs restored in this run
    -- fingerprint of the six canonical metrics, for dedup (§4.2):
    metrics_hash  TEXT NOT NULL
);

-- Per-branch snapshot rows: the "richer per-branch" data the flat JSON lacked.
CREATE TABLE branch_snapshots (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id          TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    repo_id         TEXT NOT NULL REFERENCES repos(id),
    ref_name        TEXT NOT NULL,              -- e.g. 'origin/feature/x' or 'refs/heads/x'
    kind            TEXT NOT NULL,              -- 'local'|'remote'|'tag'
    tip_sha         TEXT NOT NULL,
    commit_count    INTEGER,
    last_commit_at  TEXT NOT NULL,             -- ISO-8601 UTC
    author_name     TEXT,
    author_email    TEXT,
    subject         TEXT,
    upstream        TEXT,
    ahead           INTEGER,
    behind          INTEGER,
    is_merged       INTEGER NOT NULL,           -- 0/1
    is_stale        INTEGER NOT NULL,
    is_protected    INTEGER NOT NULL,
    is_standard     INTEGER NOT NULL,
    violation_reason TEXT                        -- naming-audit message when non-standard
);

-- Backup snapshots (owned by the backup module, doc 08; recorded here because
-- history.db is the single embedded DB — CONVENTIONS §5/§6). Referenced by runs.
CREATE TABLE snapshots (
    id           TEXT PRIMARY KEY,              -- e.g. '2026-07-11T0930-a1b2'
    repo_id      TEXT NOT NULL REFERENCES repos(id),
    created_at   TEXT NOT NULL,
    trigger      TEXT NOT NULL,                 -- 'manual'|'pre-op'|'scheduled'
    ref_count    INTEGER NOT NULL,
    note         TEXT,
    verified_at  TEXT,                          -- last successful verify
    manifest_ref TEXT NOT NULL                   -- refs/gitpurge/backups/<id>/...
);

CREATE INDEX idx_runs_repo_time    ON runs(repo_id, started_at);
CREATE INDEX idx_metrics_repo_time ON metrics(repo_id, captured_at);
CREATE INDEX idx_bsnap_run         ON branch_snapshots(run_id);
CREATE INDEX idx_bsnap_repo_ref    ON branch_snapshots(repo_id, ref_name);
```

### 3.3 Metric-set fidelity
The six columns `total, active, stale, merged, unmerged, non_standard` are named and
computed identically to the legacy `branch_history_db.json` `metrics` object
(see `.docs/cleanup/branch_history_db.json`). Any legacy record maps to exactly one
`(runs, metrics)` pair, so historical trend numbers are reproducible bit-for-bit.

### 3.4 Migration from the legacy JSON

```
git-purge history import --legacy-json <path/to/branch_history_db.json> [--map backend=<id>,frontend=<id>] [--execute]
```

- Reads the top-level `{ "<repo>": [ {timestamp, commit, date_str, metrics} … ] }`.
- Ensures a `repos` row per key (creating one, or mapping to an existing tracked
  repo id via `--map`).
- For each array entry, inserts a `runs` row (`command='scan'`, `mode='execute'`,
  `started_at`=`timestamp`, `note`=`commit`/`date_str`) and a `metrics` row with the
  six values and a computed `metrics_hash`.
- **Idempotent:** existing `(repo_id, started_at, command)` rows are skipped;
  identical-metrics duplicates are dropped by `metrics_hash` (§4.2). Dry-run by
  default; `--execute` writes. Per-branch history is not reconstructable from the
  flat JSON, so `branch_snapshots` stays empty for imported runs (populated going
  forward by real scans).

## 4. Recording runs & feeding trends

### 4.1 `RunReport` → history
Every executed **Plan** produces a `RunReport` (architecture §6 step 6). The history
module persists it as: one `runs` row, one `metrics` row (post-op counts), the
`branch_snapshots` rows captured during the scan phase, and a link to the pre-op
`snapshot_id`. Read-only `scan`/`report` runs are recorded too (`mode='dry-run'` for
`scan` previews, `'execute'` for an actual scan), so trends update even without a
destructive action — matching how the legacy tool recorded a run every time reports
were regenerated.

### 4.2 Dedup logic
Ported from `track_branch_trends.py`'s duplicate guard: before inserting a `metrics`
row, compare its `metrics_hash` (hash of the six canonical metrics) to the **latest**
`metrics` row for that repo. If identical, **skip** the metrics insert (the run itself
may still be recorded, but it does not create a redundant trend point). This prevents
the flat-file problem of many identical entries when reports are regenerated without
change (see the two identical `1093/35/64/27/798` frontend rows in the legacy JSON).

### 4.3 Trend computation
`Engine::history(repo)` returns a `TrendHistory` (ordered runs + per-run metrics).
The trend report derives, for the current run: `diff_prev` vs `runs[-2]` and
`diff_base` vs `runs[0]` (or `--baseline`), each `{change, ratio}` per metric — the
same `calculate_diff` shape as the legacy script.

## 5. Report formats & the `ReportSink`

`Engine::report(repo, ReportFormat)` returns a `Report` value; a `ReportSink`
adapter renders it. Formats:

| Format | Adapter | Use | Notes |
| :--- | :--- | :--- | :--- |
| `md` (default) | `MarkdownSink` | Human / PR / docs | GitHub-flavored, matching the legacy `*_branches.md` and `branch_cleanup_progress.md` look (tables, emoji headings, `> [!TIP]`/`> [!CAUTION]` callouts). |
| `json` | `JsonSink` | Machine / CI | The full `Report` struct (`type`, `generated_at`, `repo`, `metrics`, `sections[]`, `trend?`); stable within `schema_version`. |
| `html` | `HtmlSink` | Sharing / offline | Single **self-contained** file: inlined CSS/JS, no external assets, styled to the UI's **One Dark Pro** palette (doc 07) so CLI-exported and UI-rendered reports look identical. Light/dark aware. |

### 5.1 Output locations (CONVENTIONS §5)
- Default sink target is **stdout** for `md`/`json` (pipe-friendly).
- `--out <file>` writes one file; `--out <dir>` writes
  `<repo-id>-<type>-<date>.<ext>` into it.
- Every generated report is also **archived** under
  `<data_dir>/git-purge/reports/<repo-id>/<iso-timestamp>-<type>.<ext>` for later
  retrieval by the UI Reports screen (dedup by content hash). `html` always writes a
  file (never raw to a TTY).

## 6. Example rendered snippets

### 6.1 Audit — Overall Stats (matches `generate_reports.py`)

```markdown
## 📊 Repository Overall Stats

| Metric | Count | Percentage | Description |
| --- | --- | --- | --- |
| **Total Branches** | 843 | 100.0% | All tracked branches in the local and remote mirror |
| **Active Branches** (< 1 year old) | 80 | 9.5% | Branches with commits in the last year |
| **Stale Branches** (>= 1 year old) | 19 | 2.3% | Branches with last commit older than 1 year |
| **Merged Branches** (Safe to Delete) | 44 | 5.2% | Branches fully merged into the default branch |
| **Unmerged Branches** | 467 | 55.4% | Branches with unmerged work |
| **Non-Standard Naming** | 441 | 52.3% | Branches violating the Branching Strategy standards |
```

### 6.2 Audit — Naming Violations & Prefix categorization

```markdown
## ⚠️ Naming Standards Violations (Branching Strategy)

| Branch Name | Last Commit Date | Author | Issue Identified |
| --- | --- | --- | --- |
| `origin/bug/login` | 2024-03-11 | S. Eng | Non-standard prefix `bug/` (should use `feature/`, `fix/`, or `refactor/`) |
| `origin/MTF-1204` | 2023-08-02 | A. Dev | Unknown/Non-standard prefix `MTF-1204/` |

## 🗂️ Stale Branches: Categorization by Branch Prefix / Purpose

### Features (`feature/`, `feat/`) (6 branches)
| Branch Name | Last Commit Date | Author | Merged? | Type |
| --- | --- | --- | --- | --- |
| `origin/feature/old-report` | 2024-11-02 | A. Dev | ✅ Yes | Remote |
```

### 6.3 Trend — vs previous & baseline + run log (matches `branch_cleanup_progress.md`)

```markdown
### 🔄 Compare against Previous Run
Comparing current state to run on **Fri Jul 10 15:02:26 2026 +0300**:

| Metric | Old Value | New Value | Absolute Change | Change Ratio (%) |
| :--- | :---: | :---: | :---: | :---: |
| **Total Branches** | 816 | 679 | **-137** | **-16.8%** |
| **Stale Branches (>= 1 year)** | 17 | 0 | **-17** | **-100.0%** |
| **Unmerged Branches** | 463 | 326 | **-137** | **-29.6%** |

> [!TIP]
> **Cleanup Milestone**: Stale branches have been reduced by **19** branches (**100.0%** reduction) since the baseline run!

### 📜 Run History Log
| Run Date | Total | Active | Stale | Merged | Unmerged |
| :--- | :---: | :---: | :---: | :---: | :---: |
| 2026-07-10 17:21 | 679 | 99 | 0 | 52 | 326 |
| 2026-07-10 12:02 | 816 | 82 | 17 | 43 | 463 |
| 2026-07-10 03:49 (Baseline) | 843 | 80 | 19 | 44 | 467 |
```

## 7. CLI / UI parity

Both faces call the same `Engine::report`/`Engine::history` and render the same
`Report`/`TrendHistory` values (architecture §1) — no logic diverges.

| Capability | CLI ([doc 05](05-cli-spec.md)) | UI Reports screen ([doc 06](06-ui-spec.md)) |
| :--- | :--- | :--- |
| Audit report | `git-purge report --type audit --format md\|json\|html` | Audit view; export buttons for md/json/html |
| Trend report | `git-purge report --type trend` | Trend charts (SVG) over the metric timeseries |
| Run history | `git-purge history [--metric] [--limit]` | Run-history table + baseline marker |
| Legacy import | `git-purge history import --legacy-json` | Settings → Import history |
| Export target | stdout / `--out` / archived under `reports/` | `ReportSink` → file save dialog + same archive |

The UI's One Dark Pro theming (doc 07) is exactly the palette the `html` `ReportSink`
inlines, so a report exported from the CLI is visually identical to one viewed in the
app.

## 8. Traceability

| Requirement | Covered by |
| :--- | :--- |
| **R7** (reports + historical tracking integral) | §2 report types, §3 SQLite store, §4 run recording/trends, §5 formats |
| R3 (explore stats/history/changes) | `branch_snapshots` per-ref rows, `history`, audit categorization |
| R6 (shared abstractions) | `ReportSink`/`HistoryStore` ports; CLI+UI parity (§7) |
| R9/R12 (open, One Dark Pro UI) | self-contained themed `html` export (§5), UI parity (§7) |
