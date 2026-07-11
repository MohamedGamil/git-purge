# 05 — CLI Spec (`git-purge`)

`Status: Draft` · `Owner: CLI` · `Last-updated: 2026-07-11` ·
`Related: [../delivery/CONVENTIONS.md](../delivery/CONVENTIONS.md), [02-architecture.md](02-architecture.md), [04-core-spec.md](04-core-spec.md), [06-ui-spec.md](06-ui-spec.md), [08-backup-and-restore.md](08-backup-and-restore.md), [10-reporting-and-history.md](10-reporting-and-history.md), [11-safety-model.md](11-safety-model.md), [00-vision-and-scope.md](00-vision-and-scope.md)`

> This document specifies the `git-purge` command-line interface. It is normative
> for the CLI adapter only; all behavior it invokes is defined by
> [`gitpurge-core`](04-core-spec.md). Where this doc and
> [`CONVENTIONS.md`](../delivery/CONVENTIONS.md) disagree, CONVENTIONS wins.

---

## 1. Purpose & scope

`git-purge` is a **thin `clap` v4 adapter** over `gitpurge-core::Engine`. It parses
arguments into core option structs (`ScanOptions`, `ActionFilter`, `ExecMode`,
`RestoreSpec`, …), calls one `Engine` method, and renders the result as human text
or `--json`. It contains **zero git logic** (CONVENTIONS §2/§3, architecture §1).

The CLI generalizes the legacy `.docs/cleanup` bash/python scripts
(`backup_repos.sh`, `restore_repos.sh`, `delete_*_branches.sh`,
`archive_unmerged_branches.sh`, `generate_reports.py`, `track_branch_trends.py`)
into one portable, multi-repo, provider-agnostic binary. §11 is a line-by-line
migration cheat-sheet.

## 2. Design principles

1. **Thin over the core.** One subcommand → one (or few) `Engine` call(s). No
   branching policy, classification, or git access lives here.
2. **Human-first, script-ready.** Default output is styled text (tables via
   `comfy-table`, diagnostics via `miette`). `--json` emits a stable machine
   envelope (§6) for every command that produces data.
3. **Dry-run is the default** for every mutating command (`delete`, `archive`,
   `restore`, `backup prune`, `repo remove`). Mutations require an explicit
   `--execute` (CONVENTIONS §7.1). Without it, the command prints the resolved
   **Plan** and exits `0`.
4. **Backup before destroy.** `delete`/`archive` create and verify a pre-op
   Snapshot unless `--no-backup` is passed (CONVENTIONS §7.5); a failed action
   offers auto-restore from that Snapshot (architecture §6).
5. **Respects `NO_COLOR`.** Color is auto-disabled when `NO_COLOR` is set, when
   `--no-color` is passed, or when stdout is not a TTY. `--json` is never colored.
6. **Stable, categorized exit codes** (§7), mapped 1:1 from `GitPurgeError`
   (CONVENTIONS §11), so scripts branch on failure class.
7. **Deterministic & quiet-able.** `--quiet` suppresses progress/chrome; `-v/-vv`
   raise log verbosity. Progress bars (`indicatif`) go to stderr and never
   contaminate stdout or `--json`.
8. **Never surprise.** Protected refs are never touched, tags are never deleted by
   branch ops, and destructive/unmerged operations demand a stronger confirmation
   (CONVENTIONS §7.2–7.4).

## 3. Invocation & the `git purge` alias

```
git-purge <command> [subcommand] [args] [flags]
git purge  <command> [subcommand] [args] [flags]   # identical, when on PATH
```

`git` treats any `git-<name>` executable found on `PATH` as the subcommand
`git <name>`. Because the binary is named `git-purge`, once it is on `PATH`
(directly or via `install-cli`, §8.15) `git purge scan` and `git-purge scan` are
the same process with the same args. Argument `0` is inspected only to pick the
program name shown in help; behavior is identical either way.

## 4. Global flags

These are accepted **before or after** the subcommand and apply everywhere.

| Flag | Type | Default | Meaning |
| :--- | :--- | :--- | :--- |
| `--repo <id>` | string | active repo, else error | Selects the tracked repo (id, alias, or path). Many commands accept it positionally too. |
| `--json` | bool | `false` | Emit the machine envelope (§6) instead of human text. Disables color. |
| `--no-color` | bool | `false` | Force-disable ANSI styling. Implied by `NO_COLOR` env and by non-TTY stdout. |
| `--config <path>` | path | `<config_dir>/git-purge/config.toml` | Use an alternate config file (CONVENTIONS §5). |
| `--yes`, `-y` | bool | `false` | Assume "yes" for the standard y/N confirmation. Does **not** by itself satisfy the *stronger* confirmation for unmerged/destructive ops (§4.2). |
| `--execute`, `-e` | bool | `false` | Perform mutations. Absent ⇒ dry-run. Mirrors the legacy `-e/--execute`. |
| `-v`, `-vv` | count | `0` | Verbosity. `-v` = info, `-vv` = debug/trace (`tracing`). Logs go to stderr. |
| `--quiet`, `-q` | bool | `false` | Suppress progress and non-essential chrome; errors still print. |
| `--data-dir <path>` | path | `<data_dir>/git-purge/` | Override the data root (history DB, backups, reports). Testing/portability escape hatch (CONVENTIONS §5). |

### 4.1 Precedence

`CLI flag > environment variable > config file > built-in default`.
Concretely: `--no-color` and `NO_COLOR` both disable color (either wins);
`--config`/`--data-dir` override config-derived paths; `--repo` overrides the
config's `active_repo`. `--json` overrides any human-format preference and forces
`--no-color`.

### 4.2 Confirmation model (interaction with `--execute`, `--yes`)

Aligned with the safety model (CONVENTIONS §7, [11-safety-model.md](11-safety-model.md)):

| Situation | Requirement to proceed |
| :--- | :--- |
| Dry-run (no `--execute`) | Nothing. Prints the Plan, exits `0`. |
| Non-destructive execute (merged/stale deletes, archive, restore-as-tag) | `--execute` **and** a `y/N` confirmation. `--yes` satisfies the confirmation. |
| Destructive/unmerged execute (`--include-unmerged`, `--unmerged`, restore that overwrites an existing ref) | `--execute` **and** a *stronger* confirmation: the prompt requires typing the **repo id** as a token. `--yes` alone is **not** sufficient; non-interactive use additionally requires `--force-unmerged` (delete/archive) or `--force` (restore overwrite). |
| Any execute in a non-TTY without the matching `--yes`/`--force-*` | Refused with exit `7` (safety). |

## 5. Selection & filter flags (shared)

Commands that operate over a set of branches (`scan`, `plan`, `delete`, `archive`,
`report`) accept a common **filter** group. These build a core `ActionFilter` /
`ScanOptions`. Defaults mirror the legacy scripts (CONVENTIONS §9).

| Flag | Type | Default | Maps to legacy | Meaning |
| :--- | :--- | :--- | :--- | :--- |
| `--age <spec>` | duration/date | `"1 year ago"` | `-a/--age` | Stale threshold. Accepts human spec (`"6 months ago"`, `"2 years ago"`) or ISO date. Branches with last commit **≤** cutoff are *stale*. |
| `--merged` | bool | `true` for `delete` | merged set | Include merged branches (ancestors of the default branch). |
| `--unmerged` | bool | `false` | — | Select **only** unmerged branches (strong confirmation). |
| `--include-unmerged` | bool | `false` | `-u/--include-unmerged` | Add unmerged (stale) branches to the merged set. Strong confirmation for `delete`. |
| `--local` / `--remote` | bool | both | `git branch` / `-r` | Restrict scope to local or remote refs. Default: both. |
| `--exclude <glob,...>` | csv-glob | empty | `-x/--exclude` | Comma-separated glob/substring patterns to skip (case-insensitive), e.g. `--exclude "vue3,compat,release/*"`. |
| `--protected <glob,...>` | csv-glob | config list | protected list | Extra protected patterns added to the built-in + config protected set; never selected. |
| `--standard` / `--non-standard` | bool | both | naming audit | Restrict to branches that (do not) satisfy the naming policy. |
| `--sort <key>` | enum | `age` | report ordering | `age` \| `name` \| `author` \| `commits` \| `ahead` \| `behind`. |
| `--filter <expr>` | string | none | — | Free-form predicate over classification facets, e.g. `--filter 'author=="Sam" && ahead>0'`. Evaluated by core's policy engine. |
| `--limit <n>` | u32 | unlimited | — | Cap the number of selected refs (after sort). |

**Protected refs** (never selected regardless of filters): `main`, `master`,
`develop`, `staging`, `production`, `HEAD`, plus the config protected list and any
`--protected` globs (CONVENTIONS §7.3). The default branch is auto-detected
(`origin/main`, falling back to `origin/master`), matching the legacy scripts.

## 6. Output model

### 6.1 Human output
Styled by `anstyle`/`owo-colors`; tables by `comfy-table`; errors and hints by
`miette`. Progress bars (`indicatif`) render on **stderr**. Color is on only when
stdout is a TTY and neither `--no-color` nor `NO_COLOR` is set.

### 6.2 JSON envelope (`--json`)
Every data-producing command emits a single JSON object on stdout:

```json
{
  "schema_version": "1",
  "command": "delete",
  "ok": true,
  "dry_run": true,
  "repo": "backend",
  "data": { "...": "command-specific payload" },
  "warnings": [ { "code": "protected_skipped", "message": "…", "ref": "…" } ],
  "error": null
}
```

On failure, `ok` is `false`, `data` may be partial, and `error` is a serializable
projection of `GitPurgeError` (`{ "code", "message", "hint" }`, CONVENTIONS §11).
The process still exits with the mapped code (§7). `--json` output is
newline-terminated, never colored, and stable across releases within a
`schema_version`.

### 6.3 Verbosity & quiet
`-v`/`-vv` set the `tracing` level (stderr). `--quiet` suppresses spinners,
banners, and the dry-run/execute chrome, leaving only the essential result (or
nothing on success if there is no payload). `--json` implies machine-clean stdout
regardless of `--quiet`.

## 7. Exit codes

Mapped 1:1 from `GitPurgeError` categories (CONVENTIONS §11). `clap` reserves `2`
for argument/usage errors.

| Code | Category | `GitPurgeError` variant (illustrative) | When |
| :--: | :--- | :--- | :--- |
| `0` | Success | — | Completed (including a clean dry-run). |
| `1` | Generic/unexpected | `GitPurgeError::Internal` | Uncategorized failure. |
| `2` | Usage | (clap) | Bad flags/args, unknown subcommand. |
| `3` | Config | `Config` | Missing/invalid `config.toml`, bad `--config`. |
| `4` | Repo/target not found | `RepoNotFound`, `RefNotFound` | Unknown `--repo`, path not a git repo, unknown ref/snapshot. |
| `5` | Git backend | `Git`, `Backend` | gix/git2/shell failure (fetch/push/delete). |
| `6` | Auth | `Auth`, `Credential` | Missing/invalid credentials; keychain error. |
| `7` | Safety refusal | `Safety`, `ConfirmationRequired`, `Protected` | Confirmation declined/absent, protected ref, tag-guard, non-TTY without `--yes`. |
| `8` | Backup/verify | `Backup`, `VerifyFailed` | Pre-op snapshot or verification failed (destroy aborted). |
| `9` | Partial failure | `Partial` | Executed but ≥1 per-ref action failed (report lists which). |
| `10` | Cancelled | `Cancelled` | User/`CancellationToken` aborted a long op. |

## 8. Commands

Notation: `<required>`, `[optional]`, `a|b` alternatives. All commands accept the
global flags (§4); filter commands accept the selection flags (§5).

---

### 8.1 `repo` — manage tracked repositories

Tracks local and/or remote repos in `config.toml` + the history DB (CONVENTIONS §5,
R4). Generalizes the legacy hardcoded `backend`/`frontend` paths.

```
git-purge repo add <path-or-url> [--id <id>] [--name <label>] [--default-branch <ref>] [--remote <name=url>]
git-purge repo list [--json]
git-purge repo show <id> [--json]
git-purge repo remove <id> [--execute] [--purge-backups]
```

| Sub | Flags | Default | Notes |
| :--- | :--- | :--- | :--- |
| `add` | `--id <id>` | derived from name | Canonical short id (used by `--repo`). |
| | `--name <label>` | basename | Human label. |
| | `--default-branch <ref>` | auto-detect | Overrides `origin/main`→`origin/master` detection. |
| | `--remote <name=url>` | from repo | Register an extra remote. |
| `remove` | `--execute` | dry-run | Untracks the repo (config + history rows tombstoned). |
| | `--purge-backups` | `false` | Also delete the repo's bare mirror under `backups/` (destructive; strong confirm). |

- **Purpose:** register repos so every other command can address them by `--repo`.
- **Human output:** `add`/`remove` print a one-line confirmation; `list` prints a
  table (`id`, `name`, `path/url`, `default branch`, `#branches @ last scan`);
  `show` prints repo metadata + last run summary.
- **`--json`:** `data` is the `Repo`/`[Repo]` record(s) (id, canonical_url,
  local_path, default_branch, remotes, last_run_id).

```console
$ git-purge repo add ~/work/backend --id backend --name "Backend API"
Added repo 'backend' → /home/you/work/backend (default branch: origin/main)

$ git-purge repo add git@ssh.dev.azure.com:v3/org/proj/frontend --id frontend
Added repo 'frontend' → git@ssh.dev.azure.com:… (remote-only; run `git-purge scan` to index)

$ git-purge repo list
ID        NAME          LOCATION                       DEFAULT       BRANCHES
backend   Backend API   ~/work/backend                 origin/main   679
frontend  Frontend      git@…/frontend                 origin/main   391
```

---

### 8.2 `scan` — classify branches (audit, no Plan)

Runs the classification pipeline (`Engine::scan`) — the generalized
`generate_reports.py`. Read-only; never mutates.

```
git-purge scan [--repo <id>] [selection flags] [--refresh] [--json]
```

| Flag | Type | Default | Notes |
| :--- | :--- | :--- | :--- |
| `--refresh` | bool | `true` | Fetch + prune before classifying (legacy `git fetch -p --all`). `--no-refresh` to skip. |
| (selection §5) | | | Filters/sort the audit view; does not select for deletion. |

- **Purpose:** compute per-branch **Classification** (merged/unmerged, local/remote,
  stale/active, protected, standard/non-standard, ahead/behind) and repo-level
  metrics (`total`, `active`, `stale`, `merged`, `unmerged`, `non_standard`).
  Records a `scan` **Run** in history (feeds trends, R7).
- **Human output:** a summary stats block (the audit's "Overall Stats") plus a
  branch table respecting `--sort`/filters.
- **`--json`:** `data = { metrics, branches: [ {ref, kind, tip, last_commit_at,
  author, ahead, behind, merged, stale, protected, standard, violation_reason} ] }`.

```console
$ git-purge scan --repo backend
Scanning backend (fetch+prune)… 679 refs
Metrics  total 679 · active 99 · stale 0 · merged 52 · unmerged 326 · non-standard 339
BRANCH                          AGE     MERGED  STD   AHEAD/BEHIND  AUTHOR
feature/old-report              1y 2m   yes     yes   0/412         A. Dev
bugfix/legacy-thing             2y 0m   no      no    7/510         S. Eng
…
```

---

### 8.3 `plan` — show what `delete`/`archive` WOULD do (dry-run)

Resolves an `ActionFilter` into a **Plan** without executing. This is what
`delete`/`archive` print in their default (no-`--execute`) mode; `plan` lets you
inspect it independently.

```
git-purge plan [--repo <id>] [--action delete|archive] [selection flags] [--json]
```

| Flag | Type | Default | Notes |
| :--- | :--- | :--- | :--- |
| `--action <a>` | enum | `delete` | Which action's Plan to compute (`delete` or `archive`). |
| (selection §5) | | | Same filters the action would use. |

- **Purpose:** preview the exact set of Actions (per-ref) the tool would take, with
  the reason each ref was selected or skipped (protected, tag-guarded, excluded).
- **Human output:** a Plan table (`ref`, `action`, `reason`, `last commit`,
  `merged?`) + a footer count and the backup that *would* be created.
- **`--json`:** `data = { action, would_backup, items: [ {ref, op, selected,
  skip_reason?} ], counts }`.

```console
$ git-purge plan --repo backend --include-unmerged --age "2 years ago"
Plan (dry-run) · action=delete · age≤2 years ago · pre-op backup: WOULD create
DELETE (merged)    origin/feature/old-report        merged, 1y stale         [select]
DELETE (unmerged)  origin/bugfix/legacy-thing       UNMERGED, 2y stale       [select ⚠]
SKIP  (protected)  origin/main-legacy               protected pattern         —
14 to delete (11 merged, 3 unmerged) · 0 tags (guarded) · run with --execute
```

---

### 8.4 `backup` — snapshot management

Implements the bare-mirror-per-repo + namespaced snapshot-ref model
(CONVENTIONS §6, [08-backup-and-restore.md](08-backup-and-restore.md)). Generalizes
`backup_repos.sh` but space-shared: each snapshot writes refs into
`refs/gitpurge/backups/<snapshot-id>/…` inside the one bare repo.

```
git-purge backup create [--repo <id>] [--trigger manual|pre-op|scheduled] [--refs <glob,...>] [--note <text>] [--json]
git-purge backup list   [--repo <id>] [--json]
git-purge backup show   <snapshot-id> [--json]
git-purge backup verify <snapshot-id> [--json]
git-purge backup prune  [--repo <id>] [--keep <n>] [--older-than <spec>] [--execute] [--json]
```

| Sub | Flag | Type | Default | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `create` | `--trigger` | enum | `manual` | Recorded in the snapshot manifest. |
| | `--refs <glob,...>` | csv-glob | all heads+remotes | Limit captured refs. |
| | `--note <text>` | string | — | Free-text label. |
| `prune` | `--keep <n>` | u32 | config | Keep the newest N snapshots. |
| | `--older-than <spec>` | duration | — | Prune snapshots older than spec. |
| | `--execute` | bool | dry-run | Prune is destructive → dry-run default. |

- **`create` purpose:** verified point-in-time Snapshot; `create` is **not**
  gated by `--execute` (it is non-destructive/additive) but honors `--json`.
- **`verify`:** re-reads every captured ref and checks tip SHAs/object reachability;
  exit `8` on mismatch.
- **Human output:** `create` prints snapshot id, ref count, trigger, and disk delta;
  `list` a table (`id`, `created`, `trigger`, `#refs`, `note`); `show` the manifest;
  `verify` a per-ref OK/FAIL summary.
- **`--json`:** the `Snapshot`/`[Snapshot]` record(s) and, for `verify`, a
  `{ verified, failures: [...] }` payload.

```console
$ git-purge backup create --repo backend --note "before big cleanup"
Snapshot 2026-07-11T0930-a1b2 created · 679 refs captured · +12 MiB (shared ODB)
Manifest: refs/gitpurge/backups/2026-07-11T0930-a1b2/… · verified OK

$ git-purge backup list --repo backend
ID                       CREATED            TRIGGER   REFS   NOTE
2026-07-11T0930-a1b2     2026-07-11 09:30   manual    679    before big cleanup
2026-07-10T2000-9f7c     2026-07-10 20:00   pre-op    816    (auto: delete)
```

---

### 8.5 `delete` — delete stale/merged branches (safe)

The generalized `delete_backend_branches.sh`. Selects branches per §5, creates and
verifies a pre-op Snapshot (unless `--no-backup`), then deletes with auto-restore on
failure (architecture §6). Dry-run default.

```
git-purge delete [--repo <id>] [selection flags]
                 [--execute] [--yes] [--include-unmerged | --unmerged]
                 [--no-backup] [--force-unmerged] [--continue-on-error] [--json]
```

| Flag | Type | Default | Maps to legacy | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `--execute` | bool | dry-run | `-e/--execute` | Perform deletions. |
| `--age <spec>` | dur/date | `"1 year ago"` | `-a/--age` | Staleness cutoff. |
| `--merged` | bool | `true` | (default set) | Delete merged stale branches. |
| `--include-unmerged` | bool | `false` | `-u/--include-unmerged` | Also delete unmerged stale (⚠ strong confirm). |
| `--unmerged` | bool | `false` | — | Delete **only** unmerged stale (⚠). |
| `--no-backup` | bool | `false` | — | Skip the pre-op snapshot (CONVENTIONS §7.5 opt-out). |
| `--force-unmerged` | bool | `false` | — | Non-interactive token bypass for unmerged execute (§4.2). |
| `--continue-on-error` | bool | `false` | — | Don't stop on first failed delete; report per-ref outcomes (exit `9`). |

- **Purpose:** reproduce the legacy delete flow (merged-stale by default; optional
  unmerged) with a mandatory backup net and typed protection guards. Tags are never
  deleted (CONVENTIONS §7.4); protected refs never selected.
- **Human output (dry-run):** the delete Plan (§8.3). **(execute):** progress per
  ref, then a summary (`deleted`, `skipped`, `failed`) and the Snapshot id used.
- **`--json`:** `data = { snapshot_id, items:[{ref, result:"deleted|failed|skipped",
  error?}], counts, dry_run }`.

```console
# dry-run (default) — mirrors the old script's preview
$ git-purge delete --repo backend
Plan (dry-run) · merged + stale (≤1 year ago) · pre-op backup WOULD be created
[DRY-RUN] would delete origin/feature/old-report  (2024-11-02 by A. Dev, merged)
… 11 branches · run with --execute to apply

# execute merged-only (standard confirmation)
$ git-purge delete --repo backend --execute
About to delete 11 merged stale branches in 'backend'. A backup will be created first.
Proceed? [y/N] y
Backup 2026-07-11T0935-c3d4 created & verified (11 refs)
Deleting… ██████████ 11/11
Done. deleted 11 · skipped 0 · failed 0 · restore point: 2026-07-11T0935-c3d4

# execute including unmerged (stronger confirmation)
$ git-purge delete --repo backend --include-unmerged --age "2 years ago" --execute
⚠ 3 of 14 branches are UNMERGED — deleting them can lose work.
Type the repo id 'backend' to confirm: backend
Backup 2026-07-11T0940-e5f6 created & verified (14 refs)
Done. deleted 14 · failed 0 · restore point: 2026-07-11T0940-e5f6
```

---

### 8.6 `archive` — merge unmerged branches into a legacy branch

The generalized `archive_unmerged_branches.sh`. Merges stale **unmerged** branches
into a target legacy branch using an `ours`/`theirs` strategy, preserving history
before deletion. Dry-run default.

```
git-purge archive [--repo <id>] [selection flags]
                  [--target <branch>] [--strategy ours|theirs]
                  [--push] [--execute] [--yes] [--json]
```

| Flag | Type | Default | Maps to legacy | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `--target <branch>` | string | `main-legacy` | `-t/--target` | Legacy archive branch; created from default if absent. |
| `--strategy <s>` | enum | `ours` | `-s/--strategy` | `ours` (discard incoming diff, keep history) or `theirs` (take incoming content on conflict). |
| `--exclude <glob,...>` | csv-glob | empty | `-x/--exclude` | Skip matching branches (e.g. `vue3,compat`). |
| `--age <spec>` | dur/date | `"1 year ago"` | `-a/--age` | Staleness cutoff. |
| `--push` | bool | `false` (execute) | (implicit push) | Push the target branch to origin after archiving. |
| `--execute` | bool | dry-run | `-e/--execute` | Perform merges. |

- **Purpose:** aggregate abandoned unmerged work into one branch so it can be safely
  deleted later. Git hooks are bypassed internally (legacy `HUSKY=0` / hooksPath),
  and a clean worktree is ensured between merges — but via the core `GitBackend`, not
  shell-outs. The `--repo both` legacy convenience is expressed by running the
  command per tracked repo (or a config repo-group).
- **Human output (dry-run):** per branch, `[DRY-RUN] would merge <name> [strategy]
  (<last commit>)`. **(execute):** per-merge progress, then merged count and push
  result.
- **`--json`:** `data = { target, strategy, items:[{ref, merged:bool, error?}],
  pushed, counts }`.

```console
$ git-purge archive --repo frontend --strategy theirs --exclude "vue3,compat"
[DRY-RUN] would merge origin/feature/x [theirs] (2023-05-01 by S. Eng: WIP loader)
Skipping excluded: origin/upgrade/vue3-poc
… 42 branches would be archived into main-legacy

$ git-purge archive --repo frontend --strategy theirs --exclude "vue3,compat" --execute --push
Merging 42 branches into main-legacy (theirs)… ██████████ 42/42
Archived 42 · failed 0 · pushed main-legacy → origin
```

---

### 8.7 `restore` — restore a branch or tag from a snapshot

The generalized `restore_repos.sh`, snapshot-aware. Reads a backup ref and recreates
a branch (default) or a tag (`--as-tag`), never force-overwriting an existing ref
without explicit consent (CONVENTIONS §6, R2). Dry-run default.

```
git-purge restore <snapshot-id> <ref-or-glob> [--repo <id>]
                  [--as-tag] [--as <new-name>] [--target local|remote]
                  [--execute] [--yes] [--force] [--json]
```

| Flag | Type | Default | Maps to legacy | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `<snapshot-id>` | id | — | (backup source) | Snapshot to restore from; `latest` allowed. |
| `<ref-or-glob>` | ref/glob | — | — | Ref(s) to restore, e.g. `feature/x` or `'refs/heads/*'`. |
| `--as-tag` | bool | `false` | — | Recreate as a tag instead of a branch (R2/§10). |
| `--as <new-name>` | string | original | — | Restore under a different name. |
| `--target <t>` | enum | `local` | `-t/--target` | `local` (into the working repo) or `remote` (push to origin). |
| `--execute` | bool | dry-run | `-e/--execute` | Perform restore. |
| `--force` | bool | `false` | — | Overwrite an existing ref (strong confirm; never implicit). |

- **Purpose:** recover deleted/lost branches from a Snapshot, to local or remote,
  as a branch or tag, with overwrite protection. This is also the auto-restore path
  offered when a `delete`/`archive` fails.
- **Human output (dry-run):** the exact ref mapping that would be created
  (`snapshot:ref → target:name (branch|tag)`). **(execute):** per-ref result.
- **`--json`:** `data = { snapshot_id, target, items:[{src_ref, dst_ref, kind,
  result, error?}] }`.

```console
$ git-purge restore latest 'feature/old-report' --repo backend
[DRY-RUN] would restore 2026-07-11T0935-c3d4:refs/heads/feature/old-report
          → local refs/heads/feature/old-report (branch)

$ git-purge restore latest 'feature/old-report' --repo backend --execute
Restored feature/old-report → local (branch) from 2026-07-11T0935-c3d4

# restore everything to the remote server (legacy `restore_repos.sh -e -t remote`)
$ git-purge restore latest 'refs/heads/*' --repo backend --target remote --execute --yes
Restored 11 branches → origin
```

---

### 8.8 `diff` — compare two refs

```
git-purge diff <refA> <refB> [--repo <id>] [--stat|--name-only|--patch] [--json]
```

| Flag | Type | Default | Notes |
| :--- | :--- | :--- | :--- |
| `--stat` | bool | `true` | Summary (files changed, +/-). |
| `--name-only` | bool | `false` | List changed paths only. |
| `--patch`, `-p` | bool | `false` | Full unified diff. |

- **Purpose:** compare branches/commits to decide what to keep (R3). `refA`/`refB`
  are `RefSpec`s (branch, tag, SHA, or `snapshot:ref`).
- **Human output:** diffstat or patch. **`--json`:** `data = { ahead, behind,
  files:[{path, added, removed, status}], patch? }`.

```console
$ git-purge diff origin/main origin/feature/old-report --repo backend --stat
 12 files changed, 340 insertions(+), 87 deletions(-)  · ahead 0 / behind 412
```

---

### 8.9 `show` — view repo/file content at a ref or commit

```
git-purge show <ref> [<path>] [--repo <id>] [--json]
```

- **Purpose:** view the tree at a ref/commit, or a single file's content as of that
  commit (R1/R4 "view repo contents as of any commit"). `<ref>` is a `RefSpec`
  (incl. `snapshot:ref`); `<path>` optional.
- **Human output:** without `<path>`, a tree listing + commit header; with `<path>`,
  the file contents (syntax-plain). **`--json`:** `data = { commit:{sha, author,
  date, subject}, entries?:[{path, mode, kind}], content? }`.

```console
$ git-purge show origin/feature/old-report src/report.rs --repo backend
commit a1b2c3 · 2024-11-02 · A. Dev · "tweak report layout"
--- src/report.rs (2,104 bytes) ---
pub fn render(…) { … }
```

---

### 8.10 `report` — generate audit + trend reports

Full behavior in [10-reporting-and-history.md](10-reporting-and-history.md). Ports
`generate_reports.py` (audit) and `track_branch_trends.py` (trend/progress).

```
git-purge report [--repo <id>] [--type audit|trend|both]
                 [--format md|json|html] [--out <path>] [--baseline <run-id>] [selection flags]
```

| Flag | Type | Default | Notes |
| :--- | :--- | :--- | :--- |
| `--type <t>` | enum | `both` | `audit` (classify/naming/categorization), `trend` (diffs vs previous+baseline), or `both`. |
| `--format <f>` | enum | `md` | `md` (GitHub-flavored), `json` (machine), `html` (self-contained, One Dark Pro). |
| `--out <path>` | path | stdout | File or dir; defaults to stdout for `md`/`json`. For `html`/`--out DIR`, a file is written under it. Persisted copies also land in `<data_dir>/git-purge/reports/<repo-id>/`. |
| `--baseline <run-id>` | id | first recorded run | Trend baseline (else earliest run). |

- **Human/`md` output:** the audit sections (overall stats, recommended-for-deletion,
  naming violations, categorization by age/prefix/purpose) and/or trend tables
  (vs previous, vs baseline, run-history log) — see doc 10 for the rendered look.
- **`--json`:** the `Report` structure (`{ type, generated_at, metrics, sections,
  trend? }`). `html` is a single self-contained file (inlined CSS/JS).

```console
$ git-purge report --repo backend --type both --format md --out ./reports/
Wrote ./reports/backend-audit-2026-07-11.md and ./reports/backend-trend-2026-07-11.md
(also archived under <data_dir>/git-purge/reports/backend/)
```

---

### 8.11 `history` — trend history

```
git-purge history [--repo <id>] [--limit <n>] [--metric <name>] [--since <spec>] [--json]
```

| Flag | Type | Default | Notes |
| :--- | :--- | :--- | :--- |
| `--limit <n>` | u32 | 20 | Most recent N runs. |
| `--metric <name>` | enum | all | One of `total|active|stale|merged|unmerged|non_standard`. |
| `--since <spec>` | date/dur | all | Restrict to runs after a point in time. |

- **Purpose:** print the recorded run-history / metric timeseries from the SQLite
  history store (supersedes the flat `branch_history_db.json`; doc 10, R7).
- **Human output:** the run-history log table (`run date`, `total`, `active`,
  `stale`, `merged`, `unmerged`) with the baseline marked. **`--json`:** `data = {
  runs:[{run_id, at, command, mode, metrics}] }`.

```console
$ git-purge history --repo backend --limit 4
RUN DATE            TOTAL  ACTIVE  STALE  MERGED  UNMERGED
2026-07-10 17:21    679    99      0      52      326
2026-07-10 12:02    816    82      17     43      463
2026-07-10 05:04    824    82      17     43      467
2026-07-10 03:49*   843    80      19     44      467      (* baseline)
```

---

### 8.12 `auth` — credential management

Detail in [09-authentication.md](09-authentication.md). Secrets stored via `keyring`
(OS keychain) with an encrypted-file fallback (CONVENTIONS §5, R5).

```
git-purge auth add    [--repo <id>|--host <host>] [--method ssh|https|token]
                      [--username <u>] [--key <path>] [--token-stdin]
git-purge auth list   [--json]
git-purge auth remove <id>
git-purge auth test   [--repo <id>|--host <host>] [--json]
```

| Sub | Flag | Notes |
| :--- | :--- | :--- |
| `add` | `--method` | `ssh` (key/agent), `https` (user+pass), `token` (PAT). |
| | `--key <path>` | SSH identity; falls back to system SSH agent/identity if omitted. |
| | `--token-stdin` | Read token from stdin (never from argv/logs). |
| `test` | — | Attempt an authenticated `ls-remote`/fetch; report the method used. |

- **Human output:** `list` a table (`id`, `host/repo`, `method`, `username`, source);
  `test` prints ✓/✗ and the resolved credential source. **`--json`:** credential
  metadata only — **never** secret material.

```console
$ printf '%s' "$AZDO_PAT" | git-purge auth add --host dev.azure.com --method token --token-stdin
Stored token for dev.azure.com in OS keychain (id: az-pat-1)

$ git-purge auth test --repo frontend
✓ frontend: authenticated to git@ssh.dev.azure.com via SSH (system agent identity)
```

---

### 8.13 `ui` — launch the desktop app

```
git-purge ui [--repo <id>]
```

- **Purpose:** launch the installed Git Purge desktop app (Tauri), optionally
  pre-selecting a repo. If the desktop app is not installed, prints an install hint
  and exits `4`. The UI embeds `gitpurge-core` and runs standalone (architecture §7);
  this command is a convenience, not a dependency.

---

### 8.14 `completions` — shell completions

```
git-purge completions <bash|zsh|fish|powershell|elvish>
```

- **Purpose:** print a `clap`-generated completion script to stdout for the named
  shell. Install per shell convention, e.g.:

```console
$ git-purge completions zsh > ~/.zfunc/_git-purge
$ git-purge completions bash | sudo tee /etc/bash_completion.d/git-purge
```

---

### 8.15 `install-cli` — put `git-purge` on `PATH`

```
git-purge install-cli [--user | --system] [--dir <path>] [--force] [--execute]
```

| Flag | Type | Default | Notes |
| :--- | :--- | :--- | :--- |
| `--user` | bool | `true` | Install for the current user only. |
| `--system` | bool | `false` | Install for all users (needs elevation). |
| `--dir <path>` | path | per-OS default | Explicit install directory. |
| `--force` | bool | `false` | Overwrite an existing `git-purge` on PATH. |
| `--execute` | bool | dry-run | Dry-run prints the planned copy + PATH edit. |

- **Purpose:** the portable binary copies itself to a directory on `PATH` so both
  `git-purge` and the `git purge` subcommand alias work (§3). Dry-run by default:
  prints what it would copy and which PATH/profile it would edit.

**Placement per platform**

| OS | `--user` target | `--system` target | PATH wiring |
| :--- | :--- | :--- | :--- |
| Linux | `~/.local/bin/git-purge` | `/usr/local/bin/git-purge` | `~/.local/bin` is usually on PATH (XDG); else appends to shell profile. |
| macOS | `~/.local/bin` or `~/bin` | `/usr/local/bin` | Same profile logic; notes Homebrew prefix if detected. |
| Windows | `%LOCALAPPDATA%\Programs\git-purge\` | `%ProgramFiles%\git-purge\` | Adds the dir to the user (or machine) `Path` via the registry; advises restart of shells. |

Because git discovers `git-<name>` on PATH, no extra alias/config is written —
placing `git-purge` on PATH is sufficient for `git purge` to work everywhere.

```console
$ git-purge install-cli --user
[DRY-RUN] would copy /opt/gitpurge/git-purge → ~/.local/bin/git-purge
[DRY-RUN] ~/.local/bin already on PATH — no profile edit needed
Run with --execute to apply.

$ git-purge install-cli --user --execute
Installed → ~/.local/bin/git-purge · `git purge` alias active (PATH already includes it)
```

## 9. Legacy script → `git-purge` cheat-sheet

| Legacy invocation | `git-purge` equivalent |
| :--- | :--- |
| `backup_repos.sh` | `git-purge backup create --repo <id>` (per repo; space-shared mirror) |
| `delete_backend_branches.sh` | `git-purge delete --repo backend` (dry-run) |
| `delete_backend_branches.sh --execute` | `git-purge delete --repo backend --execute` |
| `delete_backend_branches.sh -e -u` | `git-purge delete --repo backend --include-unmerged --execute` |
| `delete_backend_branches.sh --execute -u -a "2 years ago"` | `git-purge delete --repo backend --include-unmerged --age "2 years ago" --execute` |
| `delete_frontend_branches.sh --execute` | `git-purge delete --repo frontend --execute` |
| `restore_repos.sh -t local` | `git-purge restore latest 'refs/heads/*' --repo <id> --target local` (dry-run) |
| `restore_repos.sh --execute -t local` | `git-purge restore latest 'refs/heads/*' --repo <id> --target local --execute` |
| `restore_repos.sh --execute -t remote` | `git-purge restore latest 'refs/heads/*' --repo <id> --target remote --execute` |
| `archive_unmerged_branches.sh --repo both` | `git-purge archive --repo backend` then `--repo frontend` (dry-run) |
| `archive_unmerged_branches.sh -r backend -s ours -e` | `git-purge archive --repo backend --strategy ours --execute` |
| `archive_unmerged_branches.sh -r frontend -s theirs -x "vue3,compat" -e` | `git-purge archive --repo frontend --strategy theirs --exclude "vue3,compat" --execute` |
| `generate_reports.py` | `git-purge report --repo <id> --type audit --format md` |
| `regenerate_reports.sh` | `git-purge scan --repo <id> --refresh && git-purge report --repo <id> --type audit` |
| `track_branch_trends.py` | `git-purge report --repo <id> --type trend` (or `git-purge history`) |

## 10. Terminal session transcripts

### 10.1 Core flow — scan → plan → backup → delete → restore

```console
$ git-purge scan --repo backend
Scanning backend (fetch+prune)… 843 refs
Metrics  total 843 · active 80 · stale 19 · merged 44 · unmerged 467 · non-standard 441
19 stale branches (11 merged, 8 unmerged). Run `git-purge plan` to preview cleanup.

$ git-purge plan --repo backend
Plan (dry-run) · action=delete · merged + stale (≤1 year ago) · pre-op backup WOULD be created
DELETE (merged)  origin/feature/old-report     2024-11-02  A. Dev
DELETE (merged)  origin/fix/legacy-typo         2024-09-18  S. Eng
… 11 to delete · 0 unmerged (use --include-unmerged) · 0 tags (guarded)

$ git-purge backup create --repo backend --note "pre cleanup 2026-07-11"
Snapshot 2026-07-11T0930-a1b2 created · 843 refs · +9 MiB (shared ODB) · verified OK

$ git-purge delete --repo backend --execute
About to delete 11 merged stale branches in 'backend'. A backup will be created first.
Proceed? [y/N] y
Backup 2026-07-11T0931-b2c3 created & verified (843 refs)
Deleting… ██████████ 11/11
Done. deleted 11 · skipped 0 · failed 0 · restore point: 2026-07-11T0931-b2c3

# oops — one of those was still needed
$ git-purge restore 2026-07-11T0931-b2c3 'feature/old-report' --repo backend --execute
Restored feature/old-report → local (branch) from 2026-07-11T0931-b2c3
```

### 10.2 Dangerous path — unmerged delete with strong confirmation, scripted

```console
$ git-purge delete --repo frontend --include-unmerged --age "3 years ago" --execute --yes --force-unmerged --json --quiet
{"schema_version":"1","command":"delete","ok":true,"dry_run":false,"repo":"frontend",
 "data":{"snapshot_id":"2026-07-11T0950-d4e5",
   "items":[{"ref":"origin/feature/abandoned","result":"deleted"},
            {"ref":"origin/bugfix/ancient","result":"deleted"}],
   "counts":{"deleted":2,"failed":0,"skipped":0}},
 "warnings":[],"error":null}
```

### 10.3 Reporting & trends

```console
$ git-purge report --repo backend --type both --format md --out ./reports/
Wrote ./reports/backend-audit-2026-07-11.md
Wrote ./reports/backend-trend-2026-07-11.md

$ git-purge history --repo backend --limit 4
RUN DATE            TOTAL  ACTIVE  STALE  MERGED  UNMERGED
2026-07-10 17:21    679    99      0      52      326
2026-07-10 12:02    816    82      17     43      463
2026-07-10 05:04    824    82      17     43      467
2026-07-10 03:49*   843    80      19     44      467      (* baseline)
```

## 11. Traceability

| Requirement | Covered by |
| :--- | :--- |
| R1 (analyze tooling; local+remote; view content at commit) | `scan`, `show`, `diff`, `--repo` |
| R2 (backup, restore-as-branch/tag, never force, auto-restore) | `backup`, `delete`/`archive` pre-op backup, `restore` |
| R3 (explore/filter/sort/compare/diff) | selection flags (§5), `scan`, `plan`, `diff`, `history` |
| R4 (track local+remote repos) | `repo`, `--repo` |
| R5 (auth methods, secure storage) | `auth` |
| R6 (shared core) | every command is a thin `Engine` call (§2) |
| R7 (reports + history) | `report`, `history`, `scan` run recording |
| R9/R10 (portable single binary) | `install-cli`, `completions`, no runtime deps |
