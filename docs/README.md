# Git Purge — Documentation Index

**Git Purge** is a CLI-first, safety-first utility for cleaning up old and stale
branches in Git repositories, with an optional Tauri desktop UI that drives the
exact same core logic. This directory holds the product specs and design docs.

- Canonical decisions live in [`../delivery/CONVENTIONS.md`](../delivery/CONVENTIONS.md) — **read that first.**
- The phased implementation plan and agent-runnable tasks live in [`../delivery/`](../delivery/).

## Reading order

| # | Doc | What it covers |
| :- | :--- | :--- |
| 00 | [Vision & Scope](00-vision-and-scope.md) | Problem, goals, non-goals, personas, success criteria |
| 01 | [Tech Stack](01-tech-stack.md) | Recommended stack beyond Tauri, with rationale & alternatives |
| 02 | [Architecture](02-architecture.md) | Crate layout, layering, shared-core model, data flow |
| 03 | [Domain Model](03-domain-model.md) | Entities, classification, policies, state |
| 04 | [Core Library Spec](04-core-spec.md) | `gitpurge-core` API surface, `GitBackend` trait |
| 05 | [CLI Spec](05-cli-spec.md) | Commands, flags, output formats, exit codes |
| 06 | [UI Spec](06-ui-spec.md) | Tauri v2 + Vue 3 app, screens, IPC commands, events |
| 07 | [UI Design System](07-ui-design-system.md) | One Dark Pro + Material tokens, theming, components |
| 08 | [Backup & Restore](08-backup-and-restore.md) | Snapshot model, minimal-space storage, restore flows |
| 09 | [Authentication](09-authentication.md) | SSH/HTTPS/token, secure storage, system-key fallback |
| 10 | [Reporting & History](10-reporting-and-history.md) | Reports, trend DB, exports |
| 11 | [Safety Model](11-safety-model.md) | Dry-run, confirmations, guards, backup-before-destroy |
| 12 | [Testing Strategy](12-testing-strategy.md) | Unit, integration, CLI snapshot, UI e2e, fixtures |
| 13 | [Distribution & CI](13-distribution-and-ci.md) | Tarball, Tauri bundles, GitHub Actions release |
| 14 | [Security](14-security.md) | Threat model, secrets, supply chain, hardening |
| 15 | [Extensibility](15-extensibility.md) | Provider/VCS abstractions, plugin seams |
| — | [Roadmap](ROADMAP.md) | Phases, milestones, estimates, dependency graph |
| — | [ADRs](adr/) | Architecture Decision Records |

## Prior art

Git Purge productizes and generalizes a set of hand-written bash/python scripts
(`backup_repos.sh`, `restore_repos.sh`, `delete_*_branches.sh`,
`archive_unmerged_branches.sh`, `generate_reports.py`, `track_branch_trends.py`)
that cleaned two hardcoded repos. Git Purge turns that workflow —
**classify → backup → act → report → restore-on-failure** — into a portable,
cross-platform, multi-repo, provider-agnostic tool with a UI. See
[00-vision-and-scope.md](00-vision-and-scope.md#prior-art) for the mapping.
