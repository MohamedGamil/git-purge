# Git Purge User Guide

`Last-updated: 2026-07-12` · Related: [README.md](README.md)

Welcome to the user guide for **Git Purge**. This document describes how to install, configure, and use Git Purge to safely clean up stale, merged, and abandoned branches in your Git repositories.

---

## Table of Contents
1. [Core Concepts](#1-core-concepts)
2. [Installation](#2-installation)
3. [CLI Reference & Commands](#3-cli-reference--commands)
4. [Desktop UI Guide](#4-desktop-ui-guide)
5. [Safety Nets & Protected Refs](#5-safety-nets--protected-refs)
6. [Authentication Setup](#6-authentication-setup)

---

## 1. Core Concepts

Git Purge is built around a simple, safe pipeline: **Scan ➔ Plan ➔ Backup ➔ Delete/Archive ➔ Report ➔ Restore (on error)**.

*   **Classification**: When scanning, Git Purge classifies every branch into facets:
    *   **Merged / Unmerged**: Checks if the branch's tip commit has been integrated into the default branch (e.g., `main` or `master`).
    *   **Stale / Active**: Measures elapsed time since the branch's last commit. Thresholds can be customized (e.g., branches older than 30 days are stale).
    *   **Protected**: Branches containing critical work or history (e.g., `main`, `develop`) which cannot be modified.
    *   **Standard / Non-Standard**: Checks whether the branch name follows configured naming policies (e.g., `feature/*`, `bugfix/*`).
    *   **Ahead / Behind**: Computes how many commits the branch is ahead or behind its tracking upstream.
*   **The Dry-Run Plan**: Every mutating action requires an explicit `--execute` flag. By default, Git Purge runs in dry-run mode, outputting a precise execution plan of what *would* happen.
*   **Minimal Space Backup Snapshots**: Before deleting or archiving, Git Purge backs up target branches. Unlike a traditional clone, it pushes references to a shared bare git mirror under namespaced refs (`refs/gitpurge/backups/...`). This ensures snapshots share the underlying object database and consume minimal extra disk space.

---

## 2. Installation

### 2.1 CLI Installation
Download the appropriate tarball for your platform from the GitHub releases page:

```bash
# Extract the binary
tar -xzf git-purge-v0.3.1-linux-x86_64.tar.gz
cd git-purge-v0.3.1-linux-x86_64

# Install the binary to your user PATH
./git-purge install-cli --user
```

Once installed and added to your `PATH`, Git Purge can be run as `git-purge` or as a Git subcommand: `git purge`.

### 2.2 Desktop UI Installation
Native installers are provided for major platforms:
*   **Linux**: `.deb`, `.rpm`, `.AppImage`
*   **macOS**: `.dmg` (drag-and-drop bundle)
*   **Windows**: `.msi` and `.exe` installers

---

## 3. CLI Reference & Commands

The CLI structure is hierarchical: `git-purge <command> [subcommand] [flags]`.

### 3.1 Managing Tracked Repositories
Git Purge operates on registered directories. 

```bash
# Add a repository
git-purge repo add ./my-project

# List all tracked repositories
git-purge repo list

# Remove a repository
git-purge repo remove <repo-id>
```

### 3.2 Scanning and Planning
Before acting, audit your repository branches:

```bash
# Scan and output raw classifications
git-purge scan --repo <id>

# Run a dry-run plan using filter rules
git-purge plan --merged --age "6 months ago"
```

Common filters include:
*   `--age <duration>`: Match branches whose last commit is older than a specified duration (e.g., "30 days ago", "1 year ago").
*   `--merged`: Target only branches that have been merged into the default branch.
*   `--include-unmerged`: Force inclusion of unmerged branches. Note that deleting these requires elevated confirmation because they represent unintegrated work.
*   `--exclude <glob>`: Exclude branches matching specific glob patterns.

### 3.3 Execution: Deletion and Archiving
When you are satisfied with the dry-run plan, append `--execute` to perform the changes:

```bash
# Safely delete merged branches older than 30 days
git-purge delete --merged --age "30 days ago" --execute
```

If any branch in the plan is **unmerged**, Git Purge will prompt you for confirmation. Alternatively, you can use the `archive` command which merges unmerged branches into a legacy tracking branch rather than discarding them:

```bash
# Archive unmerged branches instead of hard-deleting them
git-purge archive --include-unmerged --age "90 days ago" --execute
```

### 3.4 Backup & Restore Management
Backups are triggered automatically before mutations, but they can also be managed manually:

```bash
# Force-create a backup snapshot
git-purge backup create

# List snapshots
git-purge backup list

# Restore a branch 'feature/old-idea' from snapshot 'snap_1234'
git-purge restore snap_1234 feature/old-idea

# Restore a branch as a tag to avoid name collisions
git-purge restore snap_1234 feature/old-idea --as-tag
```

---

## 4. Desktop UI Guide

The Git Purge Desktop UI provides a modern, interactive dashboard built on Tauri and Vue 3.

### 4.1 Repository Manager
*   Add, list, and switch between multiple local Git repositories.
*   Shows indicators for each repository's path, active branch, and sync status.

### 4.2 Interactive Scan & Analysis Board
*   View a list of all branches categorized by facets (Merged, Unmerged, Stale, Active, Protected, Non-standard).
*   Filter branches using a built-in search bar that supports both plain text and **regular expressions**.
*   View calculated metrics such as commits ahead/behind, author name, and date of the last commit.

### 4.3 Executing Actions
*   Check checkboxes for branches you want to clean up.
*   A **Dry-Run Panel** dynamically updates to show exactly what changes will occur.
*   Click **Execute** to trigger deletion. A secure confirmation dialog appears, prompting you for consent—especially if unmerged branches are targeted.
*   Progress is shown via a loading spinner animation.

### 4.5 History and Audit Logs
*   Click the **History** tab to inspect all previous cleaning operations.
*   The SQLite-backed execution table supports pagination and can be expanded to view the full CLI command, execution parameters, and a list of branches deleted/archived.

### 4.6 Configuration Settings
*   **Theme Toggle**: Swaps between Light Mode and a custom One Dark Pro dark theme.
*   **Naming Conventions**: Define Regex patterns for allowed branch names (e.g., `^(feature|bugfix|hotfix)\/`). If a branch name doesn't match, it is flagged as "Non-Standard".
*   **Custom Date-Time Format**: Select from multiple representation formats (e.g., Absolute, Relative, or Custom ISO-8601 patterns). The chosen format persists across reboots via `config.toml`.

---

## 5. Safety Nets & Protected Refs

To ensure that developers never lose code, Git Purge enforces these safety rules:

1.  **Dry-Run by Default**: Actions require explicit `--execute` (CLI) or double-confirmation (UI).
2.  **Protected Refs Restriction**: The branch engine automatically excludes well-known refs (`main`, `master`, `develop`, `staging`, `production`, `HEAD`, plus any currently checked-out branch) from deletion. You can append custom globs to the protected refs list.
3.  **No Tag Deletions**: Deleting branches never impacts repository tags, even if a tag has the same name.
4.  **Automatic Backup Verification**: A backup snapshot is created and verified immediately before any delete/archive action. If the backup verification fails, the deletion aborts.
5.  **Rollback Offer**: If a delete operation fails midway, Git Purge offers to automatically restore all deleted branches from the pre-op snapshot.

---

## 6. Authentication Setup

When cleaning remote-tracking branches, Git Purge must authenticate with your Git hosting provider (e.g., GitHub, GitLab).

*   **OS Keychain Integration**: Git Purge integrates with your system's secure keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service via DBus) to securely store SSH passphrases or HTTPS Personal Access Tokens (PAT).
*   **Encrypted File Fallback**: If no keyring daemon is running, Git Purge falls back to a locally stored, encrypted file (`<config_dir>/git-purge/credentials.enc`) protected by a master password.

Manage credentials via the CLI:
```bash
# Add a credential
git-purge auth add https://github.com

# Test credential connection
git-purge auth test https://github.com
```
