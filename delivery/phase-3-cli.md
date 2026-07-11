# Phase 3 — Actions + CLI (delete / archive / restore)

`Status: Draft` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p3--actions--cli-12-ed), [CONVENTIONS §7/§9/§11](CONVENTIONS.md), [domain model §7](../docs/03-domain-model.md), [05-cli-spec.md](../docs/05-cli-spec.md), [11-safety-model.md](../docs/11-safety-model.md), [12-testing-strategy.md](../docs/12-testing-strategy.md)`

## Goal

Turn the read engine and backup net into safe mutations and expose the whole product
as a complete CLI. Implement the `delete` (merged/stale + optional unmerged),
`archive` (merge unmerged into a legacy branch via ours/theirs), and `restore`
actions, each wired through the full safety model and routed through P2's
auto-restore guard. Then build the `clap` CLI covering every verb in
[CONVENTIONS §9](CONVENTIONS.md): dry-run is the default, mutations require
`--execute`, destructive ops require a stronger confirmation, output supports human
tables and `--json`, plus completions and `install-cli`. The CLI reproduces every
capability of the original bash scripts.

**Milestone:** M2 (delete/archive) → M3 (CLI 1.0, with P5/P6).

**Dependencies:** **P1** (scan/plan/diff/show) and **P2** (backup/restore/guard)
merged. `auth`/`history`/`report` verbs are wired here as thin shells and fully
backed by **P5** (report/history) and **P6** (auth).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P3-T1** | `delete` action | Delete local/remote branches selected by an `ActionFilter` (merged/stale by default, unmerged only when `include_unmerged`), through the safety model: protection + tag + HEAD guards, pre-op snapshot, guarded execution. Ports `delete_*_branches.sh`. | `crates/gitpurge-core/src/action/delete.rs` | P2-T6 | no | 1.5 | On `stale_repo`: deletes exactly the merged/stale set, skips protected/HEAD; a tag with a matching name is never deleted (**SAFE-03**); pre-op snapshot exists first (**SAFE-04**). |
| **P3-T2** | `archive` action | Merge an unmerged branch into a configured legacy/archive branch using `MergeStrategy::{Ours,Theirs}`, then optionally delete the source. Ports `archive_unmerged_branches.sh`. | `crates/gitpurge-core/src/action/archive.rs` | P2-T6 | yes | 1.5 | Archiving an unmerged branch with `theirs` produces the expected merged tip on the legacy branch; source removal is gated by the same safety model; snapshot taken first (**SAFE-04**). |
| **P3-T3** | `Engine::execute` orchestration | Orchestrate a `Plan` → `RunReport`: honor `ExecMode` (DryRun default), create+verify the pre-op `Snapshot` unless `--no-backup`, run each `PlanItem` through the auto-restore guard, collect per-item `ActionOutcome`, compute `RunMetrics`. Emits progress via `ProgressSink`. | `crates/gitpurge-core/src/action/execute.rs`, `crates/gitpurge-core/src/lib.rs` | P3-T1, P3-T2 | no | 1.5 | `execute` in `DryRun` mutates nothing (**SAFE-01**); in `Execute` it snapshots→acts→records; a mid-run failure triggers offered restore for that item (**SAFE-05**) and the run continues/aborts per policy. |
| **P3-T4** | `clap` CLI skeleton + all verbs | Full `clap` v4 (derive) command tree for every verb in [CONVENTIONS §9](CONVENTIONS.md): `repo add/list/remove/show`, `scan`, `plan`, `backup create/list/show/verify/prune`, `delete`, `archive`, `restore`, `diff`, `show`, `report`, `history`, `auth add/list/remove/test`, `ui`, `install-cli`, `completions`. Global flags: `--repo --json --no-color --config --yes --execute -v/-vv --quiet`. Each subcommand maps args → core call → render. **Zero git logic in the CLI.** | `crates/gitpurge-cli/src/main.rs`, `crates/gitpurge-cli/src/cli.rs`, `crates/gitpurge-cli/src/cmd/*.rs` | P1-T9, P3-T3 | no | 2 | `git-purge --help` lists all verbs; every subcommand parses its documented flags; architecture test still forbids `gix`/`git2` in the CLI crate (**R6**). |
| **P3-T5** | Dry-run default + confirmations + exit codes | Dry-run is default for `delete`/`archive`/`restore`/`prune`; `--execute` opts in; destructive/unmerged ops require a stronger confirmation (typed token) via `dialoguer`, bypassable only with `--yes`; map `GitPurgeError` → stable exit codes with `miette` rendering. | `crates/gitpurge-cli/src/confirm.rs`, `crates/gitpurge-cli/src/exit.rs`, `crates/gitpurge-cli/src/cmd/*.rs` | P3-T4 | yes | 1.5 | `delete` without `--execute` prints the plan and exits 0 having changed nothing (**SAFE-01**); deleting unmerged without the typed token refuses; documented exit codes match on success/refusal/error. |
| **P3-T6** | Output rendering (human + JSON) | Render `ScanResult`/`Plan`/`RunReport`/`DiffResult`/`TreeView` as `comfy-table` human output (respecting `--no-color`/`NO_COLOR` via `anstyle`) and as stable `--json` for scripting. | `crates/gitpurge-cli/src/render/{mod.rs,table.rs,json.rs}` | P3-T4 | yes | 1.5 | Same `plan` renders as a readable table and as schema-stable JSON; `--no-color` strips ANSI; JSON validates against the documented shape. |
| **P3-T7** | Completions + `install-cli` | `completions <shell>` (bash/zsh/fish/powershell) generated from the `clap` tree; `install-cli [--user|--system]` places `git-purge` on `PATH` (and enables the `git purge` subcommand), no elevated rights for `--user`. Supports R10 zero-setup. | `crates/gitpurge-cli/src/cmd/completions.rs`, `crates/gitpurge-cli/src/cmd/install_cli.rs` | P3-T4 | yes | 1 | `completions zsh` emits a valid script; `install-cli --user` puts the binary on `PATH` in a temp HOME and `git purge --version` resolves; **R10**. |
| **P3-T8** | `assert_cmd` + `insta` snapshot suite | Feature-test the CLI end to end on fixtures with `assert_cmd` + `insta`: every verb's happy path, dry-run vs execute, confirmation gating, JSON output, and a named regression test per safety invariant. | `crates/gitpurge-cli/tests/{cli_snapshots.rs,safety.rs}`, `crates/gitpurge-cli/tests/snapshots/*` | P3-T5, P3-T6, P3-T7 | yes | 1.5 | Snapshot suite passes and is committed; named tests `safe_01`..`safe_06` assert dry-run/protection/tag-guard/backup/auto-restore/no-force; CLI reproduces each legacy-script capability (**R8**). |

Total ≈ 12 ED.

## Exit criteria

- `assert_cmd` + `insta` snapshot suite passes; the CLI reproduces every capability of
  the original bash scripts (ROADMAP P3 exit).
- `delete`, `archive`, `restore` all run through the full safety model: dry-run
  default, backup-before-destroy, protection/tag/HEAD guards, auto-restore on failure.
- Every verb from [CONVENTIONS §9](CONVENTIONS.md) exists with `--json`, exit codes,
  completions, and `install-cli`.

### Requirements & safety invariants satisfied

- **R2** (delete/archive/restore with safety net, end to end): P3-T1, P3-T2, P3-T3.
- **R3** (filter/sort surfaced as CLI flags): P3-T4, P3-T6.
- **R6** (CLI is a thin adapter; no git logic): P3-T4 (guarded by the P0 arch test).
- **R8** (unit + feature testing): P3-T8.
- **R10** (single-binary / portable + install): P3-T7.
- **SAFE-01** dry-run default: P3-T3, P3-T5. **SAFE-02** protection: P3-T1.
  **SAFE-03** tag guard: P3-T1. **SAFE-04** verified pre-op backup: P3-T3.
  **SAFE-05** auto-restore: P3-T3. **SAFE-06** no-force restore: P3-T1/restore path.
  Each has a named regression test in P3-T8.

## Risks & open questions

- **Remote deletes need auth (P6).** P3 tests operate on local fixture mirrors;
  authenticated remote `push --delete` is exercised once P6 lands. Wire the code path
  now behind `GitBackend`, gate the live remote tests on P6.
- **`archive` conflict handling** — ours/theirs resolves cleanly, but decide behavior
  when the legacy branch itself is protected or when a merge produces an empty diff.
- **Confirmation UX in `--json`/non-TTY** — refuse destructive ops without `--yes` when
  no TTY is present; ensure scripts fail closed, not open (SAFE-01).
- **Exit-code stability** — the code map becomes a compatibility surface; document it in
  [05-cli-spec.md](../docs/05-cli-spec.md) and snapshot-test it.
- **`auth`/`report`/`history` verbs** are shells here; ensure their `--help` and arg
  shape are stable so P5/P6 fill behavior without breaking snapshots.
