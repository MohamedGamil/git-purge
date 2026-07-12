<p align="center">
  <img src="assets/polished_app_icon.png" alt="Git Purge Logo" width="128" height="128" style="border-radius: 8px">
</p>

<h1 align="center">Git Purge</h1>

<p align="center">
<a href="https://github.com/MohamedGamil/git-purge/actions/workflows/ci.yml">
<img src="https://github.com/MohamedGamil/git-purge/actions/workflows/ci.yml/badge.svg" alt="CI status">
</a>
<a href="LICENSE">
<img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="license">
</a>
</p>

<br>

<p align="center">
<strong>Safely purge stale branches — with a safety net under every operation.</strong><br>
<em>A CLI-first Rust utility with an optional Tauri + Vue desktop UI for cleaning up stale, merged, and abandoned git branches without ever losing work.</em>
</p>

<br>
<br>

## Why Git Purge?

Long-lived repositories accumulate thousands of branches: features that shipped months ago, experiments nobody remembers, remote-tracking refs whose upstreams are long gone. Deleting them by hand is tedious; deleting them with a one-off script is *dangerous*. Git Purge makes branch cleanup routine and reversible.

### What is Git Purge?
*   **Safety-First Branch Purging** — A utility that identifies and cleans stale, merged, and abandoned branches safely.
*   **Dry-Run by Default** — Every destructive command is dry-run by default—you see exactly what *would* happen before anything changes.
*   **Point-in-Time Backup Snapshots** — Every delete/archive is preceded by a verified backup snapshot using a space-efficient, shared bare git mirror, making any mistake restorable in one step.
*   **Type-Safe Protected Refs** — Critical branches (`main`, `master`, `develop`, `staging`, `production`, `HEAD`, plus your own configurable globs) are structurally protected.
*   **Shared Engine Core** — One shared Rust core (`gitpurge-core`), two thin adapters (CLI and Tauri), and one Vue webview UI. Both CLI and Desktop call the exact same logic.

### What Git Purge is NOT
*   **Not a general Git Client** — It is not a replacement for full-featured Git GUIs (like GitKraken, lazygit, or Fork) for daily code staging, commits, or rebases.
*   **Not a Code-Review / PR tool** — It focuses solely on branch classification and cleanup; it does not review code or manage PR statuses.
*   **Not a Hosted / Multi-User Server** — It runs completely locally as a single-user desktop or CLI tool.
*   **Not a Git History Rewriter** — It does not rewrite Git history (unlike BFG or `git filter-repo`); it cleans up branch references and archives unmerged lines of work.

### Who is it Useful For?
*   **Release Engineers & Maintainers** — Who need to safely bulk-clean shared repositories while preserving a clear audit trail and generating compliance reports.
*   **Team Leads & Managers** — Who want to monitor technical debt reduction trends using the built-in SQLite history logger.
*   **Individual Developers** — Who want to keep their local clones organized and clean without the stress of running accidental destructive Git commands.

<br>

## Table of Contents

<details>
<summary>Click to expand</summary>

- [Features](#features)
- [Architecture](#architecture)
- [Install](#install)
- [Quick Start](#quick-start)
- [Build From Source](#build-from-source)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

</details>

<br>
<hr>

## Features

- **Scan & Classify** — Enumerates local/remote branches and computes facets: merged/unmerged, stale/active (age), protected, standard/non-standard naming, ahead/behind.
- **Plan (Dry-Run)** — Shows the exact set of deletes/archives a command *would* perform, each with a human-readable reason.
- **Backup Snapshots** — Space-efficient, point-in-time captures of refs into a shared bare mirror (N snapshots ≈ O(changed objects), not O(N × repo size)).
- **Safe Delete** — Removes stale/merged branches; unmerged deletes require stronger confirmation and are always backed up first.
- **Archive** — Merges unmerged branches into a legacy branch (ours/theirs) instead of dropping their work.
- **Restore** — Recreates a branch or a tag from any snapshot; never overwrites an existing ref without explicit consent. Auto-restore triggers on a failed delete.
- **Diff & Show** — Compare two refs; view a file/tree at any ref or commit.
- **Reports & Trends** — Audit reports (md/json/html) and historical trend tracking backed by SQLite.
- **Auth keychain Integration** — SSH key / HTTPS token / user-pass credentials via the OS keychain, with an encrypted-file fallback.

---

## Architecture

**One shared Rust core, two thin adapters, one webview UI.** The CLI and the desktop backend contain *zero* git logic, all behavior lives in `gitpurge-core` and is exercised identically by both.

```
                ┌────────────────────────────────────────────────┐
                │   gitpurge-core  (Rust library, all logic)     │
                │   git engine · backup · policy · report · auth │
                └────────────────────────────────────────────────┘
                     ▲                              ▲
        depends on   │                              │  depends on
        ┌────────────┴─────────┐         ┌──────────┴─────────────────┐
        │  gitpurge-cli (bin)  │         │  gitpurge-desktop (Tauri)  │
        │  clap + miette       │         │  Rust cmds ⇄ Vue 3 webview │
        └──────────────────────┘         └────────────────────────────┘
```

The desktop app is **standalone-capable**: it embeds `gitpurge-core`, so it works even if the CLI is not installed. See [`docs/02-architecture.md`](docs/02-architecture.md).

---

## Install

**Primary: portable tarball (zero-setup, no runtime dependencies).**

Download the tarball for your platform from the [latest release](https://github.com/MohamedGamil/git-purge/releases), extract it, and put the binary on your `PATH` (or let it do that for you):

```bash
tar -xzf git-purge-<version>-<target>.tar.gz
cd git-purge-<version>-<target>
./git-purge install-cli --user     # adds git-purge to your PATH
git-purge --version
```

Once on your `PATH`, it also works as a git subcommand: `git purge scan`.

**Desktop app bundles** (`.deb` / `.rpm` / `.AppImage` on Linux, `.msi` / `.exe` on Windows, `.dmg` on macOS) are attached to the same release.

---

## Quick Start

The core workflow is **scan → plan → backup → delete → restore**, and every mutating step is dry-run until you pass `--execute`:

```bash
# 1. Track a repo (or run inside one)
git-purge repo add ./my-project

# 2. See how your branches are classified
git-purge scan

# 3. Preview what a cleanup WOULD delete (dry-run — nothing changes)
git-purge plan --merged --age "1 year ago"

# 4. Take an explicit backup snapshot (delete does this automatically too)
git-purge backup create

# 5. Actually delete — requires --execute; backs up first, confirms before acting
git-purge delete --merged --age "1 year ago" --execute

# 6. Changed your mind? Restore a branch (or restore it as a tag) from a snapshot
git-purge restore <snapshot-id> feature/old-thing
git-purge restore <snapshot-id> feature/old-thing --as-tag
```

Add `--json` to any read command for machine-readable output, `--yes` to skip confirmations in automation, and `git-purge ui` to launch the desktop app.

---

## Build from Source

Requires the Rust toolchain (MSRV 1.88, pinned via `rust-toolchain.toml`). For the desktop app you also need Node 20 + pnpm 9 and your platform's Tauri prerequisites.

```bash
# CLI + core
cargo build --release
./target/release/git-purge --help

# Run the test suite
cargo nextest run --all       # or: cargo test --all

# Desktop app (dev)
cd apps/desktop
corepack enable && corepack prepare pnpm@9 --activate
pnpm install
pnpm tauri dev
```

---

## Documentation

- **User Guide:** [`docs/user-guide.md`](docs/user-guide.md) — complete guide on installation, CLI reference, Desktop UI settings, and authentication.
- **Developer Guide:** [`docs/developer-guide.md`](docs/developer-guide.md) — developer instructions, architecture layout, ports & adapters description, extension guide, and dev commands.
- **Specs & Design:** [`docs/`](docs/) — product specifications including vision, architecture, safety model, backup strategies, and technical roadmap.
- **Delivery & Conventions:** [`delivery/`](delivery/) — the canonical [`CONVENTIONS.md`](delivery/CONVENTIONS.md) (source of truth for names, versions, and safety model), the [Agent Guide](delivery/AGENT_GUIDE.md), and the [Definition of Done](delivery/DEFINITION_OF_DONE.md).

---

## Contributing

Contributions are welcome! Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) for dev setup, the local quality gates, Conventional Commits, and the PR checklist. Security issues: see [`SECURITY.md`](SECURITY.md).

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed under the Apache License 2.0, without any additional terms or conditions.
