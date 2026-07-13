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
  - [Windows](#-windows-x64)
  - [macOS](#-macos-intel--apple-silicon)
  - [Linux](#-linux-x86_64--aarch64)
  - [Git Integration](#-git-integration)
  - [Uninstall](#%EF%B8%8F-uninstall)
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

Git Purge releases are published to the [GitHub Releases Page](https://github.com/MohamedGamil/git-purge/releases) with three package choices:
1. **Combined Package:** Bundles both the CLI utility and the Desktop App installer.
2. **CLI Only:** Compact tarball/zip containing only the portable CLI binary.
3. **Desktop App Only:** System-specific installer/package for the GUI app.

---

### 💻 Windows (x64)

#### Download Options
* **Both (CLI & Desktop):** `git-purge-combined-<version>-x86_64-pc-windows-msvc.zip`
* **CLI Only:** `git-purge-<version>-x86_64-pc-windows-msvc.zip`
* **Desktop App:** `Git Purge_<version>_x64_en-US.msi` or `Git Purge_<version>_x64-setup.exe`

#### Installation & Launch
* **Desktop App:** Double-click the `.msi` or `.exe` installer and follow the prompt. Once installed, launch **Git Purge** from your Start Menu.
* **CLI Utility:** Extract the CLI `.zip` archive, open PowerShell in the extracted folder, and register it to your user PATH:
  ```powershell
  .\git-purge.exe install-cli --user
  ```
  Once the CLI is installed on your PATH, you can also launch the desktop GUI directly by running:
  ```powershell
  git-purge ui
  ```

---

### 🍏 macOS (Intel & Apple Silicon)

#### Download Options
* **Both (CLI & Desktop):** `git-purge-combined-<version>-<arch>-apple-darwin.tar.gz`
* **CLI Only:** `git-purge-<version>-<arch>-apple-darwin.tar.gz`
* **Desktop App:** `Git Purge_<version>_<arch>.dmg`

> [!NOTE]
> Choose `aarch64` for Apple Silicon (M1/M2/M3) and `x86_64` for Intel-based Macs.

#### Installation & Launch
* **Desktop App:** Mount the `.dmg` file and drag the **Git Purge** app into your `Applications` folder. You can launch it from Launchpad or Finder.
* **CLI Utility:** Extract the archive, open terminal in that folder, and run:
  ```bash
  ./git-purge install-cli --user
  ```
  Once the CLI is installed on your PATH, you can launch the desktop GUI from the terminal by running:
  ```bash
  git-purge ui
  ```

---

### 🐧 Linux (x86_64 & aarch64)

#### Download Options
* **Both (CLI & Desktop):** `git-purge-combined-<version>-x86_64-unknown-linux-gnu.tar.gz`
* **CLI Only:** `git-purge-<version>-<target>.tar.gz` (`gnu` or `musl` build)
* **Desktop App:** 
  * Debian / Ubuntu: `Git Purge_<version>_amd64.deb`
  * RedHat / Fedora: `Git Purge-<version>-1.x86_64.rpm`
  * Portable AppImage: `Git Purge_<version>_amd64.AppImage`

#### Installation & Launch
* **Desktop App:**
  * **Debian/Ubuntu:**
    ```bash
    sudo apt install ./Git_Purge_<version>_amd64.deb
    ```
    Launch from your Desktop Applications menu or by running `gitpurge-desktop` in the terminal.
  * **RedHat/Fedora:**
    ```bash
    sudo dnf install ./Git_Purge-<version>-1.x86_64.rpm
    ```
    Launch from your Desktop Applications menu or by running `gitpurge-desktop` in the terminal.
  * **AppImage:** Make executable and run:
    ```bash
    chmod +x Git_Purge_<version>_amd64.AppImage
    ./Git_Purge_<version>_amd64.AppImage
    ```
* **CLI Utility:** Extract the tarball, open your terminal in the folder, and run:
  ```bash
  ./git-purge install-cli --user
  ```
  Once the CLI is installed on your PATH, you can also launch the GUI by running:
  ```bash
  git-purge ui
  ```

---

### 💡 Git Integration

Once the CLI is installed on your `PATH`, it registers itself as a native Git subcommand. You can run it directly:
```bash
git purge scan
git purge plan --merged
git purge ui
```

---

### 🗑️ Uninstall

#### 1. Desktop GUI App
* **Windows:** Go to *Settings > Apps > Installed Apps* (or *Control Panel > Uninstall a program*), locate **Git Purge**, and click **Uninstall**.
* **macOS:** Drag the **Git Purge** application from your `Applications` folder into the **Trash**, and empty it.
* **Linux:**
  * Debian / Ubuntu: Run `sudo apt remove git-purge` (or `sudo dpkg -r git-purge`).
  * RedHat / Fedora: Run `sudo dnf remove git-purge` (or `sudo rpm -e git-purge`).
  * AppImage: Simply delete the `.AppImage` file from your system.

#### 2. CLI Utility
Simply delete the `git-purge` executable from its installation location:
* **Windows:**
  * User-level: Delete `%LOCALAPPDATA%\Programs\git-purge\git-purge.exe` and remove the directory from your PATH environment variable.
  * System-level: Delete `%ProgramFiles%\git-purge\git-purge.exe` (requires administrator privilege).
* **macOS / Linux:**
  * User-level: Delete `~/.local/bin/git-purge`.
  * System-level: Delete `/usr/local/bin/git-purge` (requires `sudo`).

#### 3. Configuration & Database Files (Optional)
To completely delete all repositories registry database, configurations, and backup snapshots:
* Delete the `~/.gitpurge/` directory (located under your home folder on all operating systems).

---

## Quick Start

The core workflow is **scan → plan → backup → delete → restore**, and every mutating step is dry-run by default until you pass `--execute` (or `-e`):

### 1. Manage Tracked Repositories
Git Purge tracks repositories locally via a SQLite-backed registry:
```bash
# Add a repository to track
git-purge repo add ./my-project --label "Backend Core"

# List all tracked repositories
git-purge repo list
```

### 2. Scan & Classify Branches
Check the status of branches in a tracked repository (runs in read-only mode):
```bash
# Scan the current repository
git-purge scan

# Scan and output in JSON format for scripting/automation
git-purge scan --json
```

### 3. Plan & Preview Cleanup (Dry-Run by Default)
Plan shows exactly what deletions or archiving actions would be performed, along with the reasons:
```bash
# Preview deleting branches merged into main that are older than 30 days
git-purge plan --merged --age "30 days ago"

# Preview archiving unmerged inactive branches older than 90 days
git-purge plan --unmerged --age "90 days ago"
```

### 4. Perform Safe Deletions & Archiving
Mutating operations require the explicit `--execute` flag. A pre-operation backup is automatically verified before any changes are applied:
```bash
# Safely delete merged branches older than 30 days (asks for confirmation)
git-purge delete --merged --age "30 days ago" --execute

# Archive unmerged branches into a legacy archive branch instead of deleting them
git-purge archive --unmerged --age "90 days ago" --execute

# Run non-interactively in CI (skips confirmation prompt)
git-purge delete --merged --yes --execute
```

### 5. Snapshot Backup & Restore
If a branch deletion or archive needs to be reverted, you can restore it from any previous point-in-time snapshot:
```bash
# List all point-in-time backup snapshots
git-purge backup list

# Re-create a deleted branch back to its original state from a snapshot
git-purge restore <snapshot-id> feature/stale-branch

# Restore a branch as a lightweight Git tag instead
git-purge restore <snapshot-id> feature/stale-branch --as-tag
```

### 6. Inspect Diffs & File History
Compare branch states and view specific file structures from history without checking out the branches:
```bash
# Compare local branches or branch history
git-purge diff main feature/stale-branch

# View a file's content at a specific ref/commit
git-purge show src/main.rs --ref main
```

### 7. Reports & History Trends
Generate HTML, Markdown, or JSON audit reports, and view technical debt reduction history:
```bash
# Generate a clean HTML audit report
git-purge report --format html --output ./reports/audit-july.html

# View cleanup run history and trends logged in SQLite
git-purge history
```

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
