# 00 — Vision & Scope

`Status: Approved` · `Owner: Product` · `Last-updated: 2026-07-11`

## Problem
Long-lived repositories accumulate hundreds or thousands of stale branches (the
source repos that motivated this tool had **843** and **2,356** branches). Cleaning
them by hand is risky: one wrong `push --delete` can destroy unmerged work. Existing
ad-hoc scripts are effective but hardcoded, single-machine, bash/python-dependent,
and unsafe for non-experts.

## Vision
**Git Purge** makes branch cleanup *safe, visible, and reversible* for anyone. It is
a portable, zero-setup CLI with an optional desktop UI, both driven by one shared
Rust core. Every destructive action is preceded by a verified backup and can be
undone; every decision is informed by clear classification, diffs, and history.

## Goals
- Safe-by-default cleanup: dry-run first, backup before destroy, one-click restore.
- Work on **local and remote** repos, across providers, cross-platform.
- Make branch state **legible**: classify, filter, sort, compare, diff, view content
  at any commit, and track trends over time.
- Ship as a **single self-contained binary** (primary: tarball) and as **desktop
  bundles** for all major OS/distros.
- One codebase, two faces (CLI + UI), extensible to new providers/VCS later.

## Non-goals (v1)
- Not a general Git GUI or a replacement for `git`/GitKraken/lazygit.
- Not a code-review/PR tool (may read PR metadata later; not v1).
- No server/hosted/multi-user mode; single-user desktop/CLI only.
- No rewriting of history inside branches (no filter-branch/BFG features).

## Personas
- **Release engineer / maintainer** — bulk-cleans a neglected repo, wants safety and
  an audit trail. Primary CLI user; uses UI for review.
- **Team lead** — wants reports/trends showing tech-debt reduction over time.
- **Individual dev** — occasional cleanup on personal repos; prefers the UI.

## Requirements traceability (R1–R12)
These IDs are referenced by phase exit criteria and the
[Definition of Done](../delivery/DEFINITION_OF_DONE.md).

| ID | Requirement (condensed) | Primary docs |
| :-- | :--- | :--- |
| **R1** | Analyze current cleanup tooling; support local + remote repos; view repo contents as of any commit. | 03, 04, 05, 06 |
| **R2** | Back up all branches before deletion (name, content, commits) as a local clone at minimal space; easily restorable; auto-restore on failed delete; user-approved restore as branch or tag; never force restore. | 08, 11 |
| **R3** | Explore branches, backups, restore points, history, commits, changes, stats; filter, sort, compare, diff branches. | 04, 05, 06, 10 |
| **R4** | Track local + remote repos; view contents at time of commit. | 04, 06, 08 |
| **R5** | Multiple git auth methods with secure storage: SSH keys, HTTPS creds, token; fallback to system SSH identity or user-provided keys/identities. | 09, 14 |
| **R6** | CLI + UI share the same core abstractions; extensible to new providers/VCS/backup/restore/diff/stats methods. | 02, 04, 15 |
| **R7** | Reports generation and historical branch tracking are integral. | 10, 05, 06 |
| **R8** | Unit and feature testing are mandatory. | 12 |
| **R9** | Open-source; distributed as a tarball (primary) + Tauri bundles for Linux distros, Windows, macOS. | 13 |
| **R10** | CLI ships as a single binary with all deps, or portable package runnable without install; restore semantics per R2. | 13, 08 |
| **R11** | GitHub Actions: on each new tag, build all targets and publish a release with attached binaries. | 13 |
| **R12** | Minimalist, intuitive UI; light/dark/system themes; Material design on the One Dark Pro palette. | 07, 06 |

## Prior art
Git Purge generalizes a custom-written bash scripts I write before (which cleaned two
hardcoded repos). The mapping to Git Purge capabilities:

| Legacy script | Git Purge home |
| :--- | :--- |
| `backup_repos.sh` (mirror clone) | `backup create` / snapshot model ([08](08-backup-and-restore.md)) |
| `restore_repos.sh` (local/remote restore, dry-run) | `restore` + safety model ([08](08-backup-and-restore.md), [11](11-safety-model.md)) |
| `delete_*_branches.sh` (merged/stale, protected, dry-run) | `delete` action ([04](04-core-spec.md), [05](05-cli-spec.md)) |
| `archive_unmerged_branches.sh` (ours/theirs to legacy) | `archive` action ([04](04-core-spec.md)) |
| `generate_reports.py` (classify, naming audit) | scan/classify + report ([04](04-core-spec.md), [10](10-reporting-and-history.md)) |
| `track_branch_trends.py` (JSON history, diffs) | history/trends (SQLite) ([10](10-reporting-and-history.md)) |

## Success criteria
- A first-time user cleans a 1,000-branch repo without data loss, with a restorable
  backup, in under 10 minutes, via CLI **or** UI.
- Zero destructive action occurs without a prior verified backup (unless explicitly
  opted out) — enforced by tests.
- Pushing a version tag produces installable artifacts for all target platforms.
