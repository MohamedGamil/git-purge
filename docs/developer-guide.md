# Git Purge Developer Guide

`Last-updated: 2026-07-12` · Related: [README.md](README.md)

This developer guide provides an overview of the codebase architecture, design decisions, core ports, and instructions on how to extend and develop **Git Purge**.

---

## 1. Architecture Overview

Git Purge is structured as a **Hexagonal Architecture (Ports and Adapters)**. The primary goal of this architecture is to decouple the core business rules from external details like the Git libraries, database implementations, OS keychains, and the presentation layers (CLI and Desktop UI).

```
                                ┌────────────────────────────────────────────────────────┐
                                │                    gitpurge-core                       │
                                │                                                        │
                                │   ┌───────────────┐        ┌──────────────────────┐    │
                                │   │ Domain Models │        │       Engines        │    │
                                │   │ (Plan, Run)   │ ◄───── │ (Scan, Backup, Action│    │
                                │   └───────────────┘        └──────────────────────┘    │
                                │                                    │                   │
                                │                  calls             ▼                   │
                                │                       ┌──────────────────────┐         │
                                │                       │      Port Traits     │         │
                                │                       │ (GitBackend, DB, ..) │         │
                                │                       └──────────────────────┘         │
                                └───────────────────────────────────▲────────────────────┘
                                                                    │
                                                           implemented by
                                                                    │
                     ┌──────────────────────────────────────────────┴──────────────────────────────────────────────┐
                     │                                              │                                              │
          ┌──────────┴──────────┐                        ┌──────────┴──────────┐                        ┌──────────┴──────────┐
          │     Git Adapter     │                        │   Database Adapter  │                        │  Keychain Adapter   │
          │ (gix, git2, shell)  │                        │       (SQLite)      │                        │  (keyring, file)    │
          └─────────────────────┘                        └─────────────────────┘                        └─────────────────────┘
                     ▲                                              ▲                                              ▲
                     │                                              │                                              │
                     └─────────────────────── drives ───────────────┼─────────────── drives ───────────────────────┘
                                                                    │
                                                        ┌───────────┴───────────┐
                                                        │  Presentation Layer   │
                                                        │  (gitpurge-cli, UI)   │
                                                        └───────────────────────┘
```

The workspace is organized as follows:
*   [crates/gitpurge-core](../crates/gitpurge-core): Contains all domain models, policies, business rules, and port traits. It has no dependencies on frontend framework components or CLI parsers.
*   [crates/gitpurge-cli](../crates/gitpurge-cli): The command-line interface. A thin adapter wrapping `gitpurge-core`.
*   [apps/desktop](../apps/desktop): The Tauri v2 desktop application. The Tauri backend acts as a thin presentation layer wrapping `gitpurge-core`, exposing IPC endpoints to the Vue 3 frontend.

---

## 2. Core Port Traits

All external services are defined as traits (ports) in `gitpurge-core`. The core engine interacts solely with these traits.

### 2.1 GitBackend
Defined in [crates/gitpurge-core/src/port/git/mod.rs](../crates/gitpurge-core/src/port/git/mod.rs). It abstracts Git operations:
*   `enumerate_branches`: Lists local and remote branches.
*   `check_merge_status`: Determines if a branch has been merged into the default branch.
*   `delete_ref` & `delete_remote_ref`: Deletes references.
*   `push_ref`: Pushes objects to a remote backup repository.

We implement a **hybrid git engine** strategy:
1.  **gix (Gitoxide)**: Primary engine for high-performance read-only actions (walking commits, counting ahead/behind).
2.  **git2 (libgit2)**: Used for authenticated network operations (pull, push, delete refs) and credential management.
3.  **ShellGitBackend**: A fallback adapter that shells out to the system `git` CLI (useful for validation and testing).

### 2.2 HistoryStore (Database)
Defined in [crates/gitpurge-core/src/port/history/mod.rs](../crates/gitpurge-core/src/port/history/mod.rs). It manages historical trend analysis and operations logging.
*   **Implementation**: SQLite adapter utilizing `rusqlite` (embedded).
*   **Location**: Managed globally under XDG/system directories (`history.db`).
*   **Capabilities**:
    *   Tracks branch counts and staleness trends over time.
    *   Records an append-only execution log.
    *   Provides paginated SQLite queries to support efficient UI navigation.

### 2.3 SecretStore (Keychain)
Defined in [crates/gitpurge-core/src/port/auth/mod.rs](../crates/gitpurge-core/src/port/auth/mod.rs). It secures authentication tokens and SSH passphrases.
*   **Implementation**: Linux DBus Secret Service, macOS Keychain, and Windows Credential Manager via the `keyring` crate.
*   **Fallback**: Encrypted credentials file (`credentials.enc`) protected by a master password when no system keyring is available.

---

## 3. Key Implementations & Enhancements

Over several development phases, several key architectural additions have been made:

### 3.1 Concurrent Scanning & Caching
To maintain high performance on massive repositories:
*   The scan phase processes branches in parallel using a thread pool.
*   Calculated facets (e.g. ahead/behind metrics, staleness) are cached in the SQLite database to avoid expensive commit-graph recalculations. The cache is automatically invalidated when ref tips change.

### 3.2 Dynamic Remote Labels & Prefix Stripping
When scanning remote repositories, Git Purge automatically detects remote structures:
*   Strips default remote prefixes (like `refs/remotes/origin/`) dynamically.
*   Supports multiple upstream remotes, classifying them under distinct remote labels.

### 3.3 Lucide SVG Icons & Custom Fonts
The Desktop UI features high-quality assets to replace standard emojis:
*   Uses native Lucide SVG components instead of system-specific emoji representations.
*   Applies a uniform dark theme modeled on One Dark Pro.
*   Integrates Google Sans as the primary body font and JetBrains Mono for metadata representations (hashes, branch names).

---

## 4. How to Extend Git Purge

### 4.1 Adding a New VCS Adapter
To add support for a new version control platform (e.g., GitHub API, GitLab API, or mercurial):
1.  Implement the `GitBackend` trait defined in `gitpurge-core::port::git`.
2.  Register the new backend inside `gitpurge-core::Engine` construction.
3.  Expose the configuration parameters inside `config.toml` (and UI forms).

### 4.2 Adding a Custom Filter / Policy
To add custom criteria for branch classification:
1.  Modify `ScanResult` and `BranchFacet` models in [crates/gitpurge-core/src/model/plan.rs](../crates/gitpurge-core/src/model/plan.rs).
2.  Implement the logic in the Policy Engine service under `gitpurge-core::service::policy`.
3.  Add matching command-line flags to [crates/gitpurge-cli/src/main.rs](../crates/gitpurge-cli/src/main.rs).
4.  Update the Desktop UI configuration interface.

---

## 5. Development & Verification Guide

### 5.1 Prerequisites
*   **Rust**: MSRV 1.88 (defined in [rust-toolchain.toml](../rust-toolchain.toml)).
*   **Node**: Version 20 LTS.
*   **pnpm**: Version 9.x.

### 5.2 Build Commands
```bash
# Build the workspace (CLI + Core + Desktop Backend)
cargo build --release

# Run Rust tests (Unit & Integration tests)
cargo test --all-targets --all-features

# Run Clippy workspace check
cargo clippy --all-targets --all-features -- -D warnings

# Build the Frontend Web Assets
cd apps/desktop
pnpm install
pnpm build

# Run Vitest Frontend Tests
pnpm test
```

### 5.3 Testkit & Fixtures
Integration tests in `gitpurge-core` use a programmatic Git repository builder (`testkit`) that generates mock repositories on the local filesystem with custom branches, commit messages, ahead/behind sequences, and merge bases. This allows tests to execute deterministically without hitting external network connections.
