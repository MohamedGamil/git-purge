<div align="center">

# Git Purge

**Safely purge stale branches — with a net under every operation.**

A CLI-first Rust utility (with an optional Tauri + Vue desktop UI) for cleaning up
stale, merged, and abandoned git branches without ever losing work.

[![CI](https://github.com/MohamedGamil/git-purge/actions/workflows/ci.yml/badge.svg)](https://github.com/MohamedGamil/git-purge/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](#license)

</div>

---

## Why Git Purge?

Long-lived repositories accumulate thousands of branches: features that shipped
months ago, experiments nobody remembers, remote-tracking refs whose upstreams are
long gone. Deleting them by hand is tedious; deleting them with a one-off script is
_dangerous_. Git Purge makes branch cleanup **routine and reversible**:

- Every destructive command is **dry-run by default** — you see exactly what _would_
  happen before anything changes.
- Every delete/archive is preceded by a **verified backup snapshot**, so a mistake is
  always one `restore` away.
- **Protected branches** (`main`, `master`, `develop`, `staging`, `production`,
  `HEAD`, plus your own list and glob patterns) are structurally excluded from
  destructive plans — not by convention, but by the type system.

## Features

| Capability | What it does |
| :--- | :--- |
| **Scan & classify** | Enumerates local/remote branches and computes facets: merged/unmerged, stale/active (age), protected, standard/non-standard naming, ahead/behind. |
| **Plan (dry-run)** | Shows the exact set of deletes/archives a command _would_ perform, each with a human "why". |
| **Backup snapshots** | Space-efficient, point-in-time captures of refs into a shared bare mirror (N snapshots ≈ O(changed objects), not O(N × repo size)). |
| **Safe delete** | Removes stale/merged branches; unmerged deletes require stronger confirmation and are always backed up first. |
| **Archive** | Merges unmerged branches into a legacy branch (ours/theirs) instead of dropping their work. |
| **Restore** | Recreates a branch _or_ a tag from any snapshot; never overwrites an existing ref without explicit consent. Auto-restore triggers on a failed delete. |
| **Diff & show** | Compare two refs; view a file/tree at any ref or commit. |
| **Reports & trends** | Audit reports (md/json/html) and historical trend tracking backed by SQLite. |
| **Auth** | SSH key / HTTPS token / user-pass credentials via the OS keychain, with an encrypted-file fallback. |
| **CLI + optional desktop UI** | Everything above is reachable from the `git-purge` CLI and the desktop app — because both call the same Rust core. |

## Architecture at a glance

**One shared Rust core, two thin adapters, one webview UI.** The CLI and the desktop
backend contain _zero_ git logic — all behavior lives in `gitpurge-core` and is
exercised identically by both.

```
                ┌───────────────────────────────────────────────┐
                │   gitpurge-core  (Rust library, all logic)     │
                │   git engine · backup · policy · report · auth │
                └───────────────────────────────────────────────┘
                     ▲                              ▲
        depends on   │                              │  depends on
        ┌────────────┴─────────┐         ┌──────────┴───────────────┐
        │  gitpurge-cli (bin)  │         │  gitpurge-desktop (Tauri) │
        │  clap + miette       │         │  Rust cmds ⇄ Vue 3 webview │
        └──────────────────────┘         └───────────────────────────┘
```

The desktop app is **standalone-capable**: it embeds `gitpurge-core`, so it works
even if the CLI is not installed. See [`docs/02-architecture.md`](docs/02-architecture.md).

## Install

**Primary: portable tarball (zero-setup, no runtime dependencies).**

Download the tarball for your platform from the
[latest release](https://github.com/MohamedGamil/git-purge/releases), extract it, and
put the binary on your `PATH` (or let it do that for you):

```bash
tar -xzf git-purge-<version>-<target>.tar.gz
cd git-purge-<version>-<target>
./git-purge install-cli --user     # adds git-purge to your PATH
git-purge --version
```

Once on your `PATH`, it also works as a git subcommand: `git purge scan`.

**Desktop app bundles** (`.deb` / `.rpm` / `.AppImage` on Linux, `.msi` / `.exe` on
Windows, `.dmg` on macOS) are attached to the same release.

## Quick start

The core workflow is **scan → plan → backup → delete → restore**, and every mutating
step is dry-run until you pass `--execute`:

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

Add `--json` to any read command for machine-readable output, `--yes` to skip
confirmations in automation, and `git-purge ui` to launch the desktop app.

## Build from source

Requires the Rust toolchain (MSRV 1.88, pinned via `rust-toolchain.toml`). For the
desktop app you also need Node 20 + pnpm 9 and your platform's Tauri prerequisites.

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

## Documentation

- **Specs & design:** [`docs/`](docs/) — vision, tech stack, architecture, domain model, roadmap.
- **Delivery & conventions:** [`delivery/`](delivery/) — the canonical
  [`CONVENTIONS.md`](delivery/CONVENTIONS.md) (source of truth for names, versions,
  and safety model), the [Agent Guide](delivery/AGENT_GUIDE.md), and the
  [Definition of Done](delivery/DEFINITION_OF_DONE.md).

## Contributing

Contributions are welcome! Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) for dev
setup, the local quality gates, Conventional Commits, and the PR checklist. Security
issues: see [`SECURITY.md`](SECURITY.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed under the Apache License 2.0,
without any additional terms or conditions.
