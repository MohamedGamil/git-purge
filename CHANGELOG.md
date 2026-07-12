# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Introduced `DESIGN.md` defining brand personality, layout grids, components, typography scales, and shape design requirements.
- Implemented and mapped design color tokens for both dark (default) and light themes.
- Enabled remote branch deletion selection, planning, and execution with premium warning prompts in both the Branches view and Plan execution screen.
- Implemented native OS save dialog integration utilizing `tauri-plugin-dialog` to bypass webview Content Security Policy (CSP) limitations on Blob downloads.
- Added branch locality (local vs remote) markers to backup references table in Backups view.
- Added an "Auto Fetch" checkbox (checked by default) in the Branches Explorer UI, core engine scan options, and CLI command flags to perform automatic remote fetch and pruning of all remotes before classifying branches.

### Changed

- Configured repository listings on the main dashboard to always sort alphabetically ascending by name.
- Styled all delete/destructive buttons with high-contrast, theme-aware danger-level red tokens (`#e06c75` in dark mode, `#ba1a1a` in light mode).
- Granted default dialog capabilities in `default.json` to authorize `ask` and `save` native dialogs, and implemented robust try-catch fallbacks to standard `confirm()` / HTML5 downloads to prevent silent promise rejection errors.
- Updated `NamingPolicy::default()` default allowed regex to include `main-legacy` as a standard branch name.
- Updated `SettingsView.vue` naming convention input placeholder and field hint to indicate that leaving it blank enforces the default naming convention.

- Implemented thread-level parallel branch classification using Rayon inside the `Scanner::classify` loop to optimize commit graph walks and ancestry checks.
- Implemented an in-memory scan cache in the central `Engine` that stores `ScanResult` metadata mapped by repository ID and is invalidated only when references/HEAD OID or policy configurations change, making repository listings and cleanup planning instantaneous when state remains unchanged.
- Rebuilt the desktop frontend theme variable configuration in `tokens.css` to define and unify color schemes using the new design token variables.
- Optimized repository list loading on the Dashboard by caching repository metrics in the trend database, avoiding redundant live Git scans and reducing list load time from seconds to sub-milliseconds.
- Added network/VPN connectivity state protections: configured global connection and operation timeouts in the libgit2 engine (under ADR-0006), implemented automatic fallback to local-only branch scans in the Tauri store on connection failure, and introduced a global offline status indicator in the UI.

### Fixed

- Resolved a bug where clearing the naming convention regex from the settings view would disable naming checks entirely (marking all branches non-standard); the core engine now correctly falls back to enforcing the default naming convention regex when the allowed list is empty.
- Fixed remote SSH connection and deletion failures by handling username request prompts (`git2::CredentialType::USERNAME`) in the credentials callback and resolving user `.ssh` directory paths dynamically without hardcoded paths.
- Configured operational logging to write logs in the user home directory under `~/.git-purge/git-purge-operations.log` using fully dynamic cross-platform path resolution.
- Fixed a critical bug in `execute_deletions_with_guard` that prevented remote branch deletions from executing by refactoring it to iterate over complete `Action` structures instead of only names, ensuring proper local vs remote branch routing.
- Fixed silent remote push rejections by registering a `push_update_reference` callback in `delete_remote_branch` to check reference status updates and propagate push errors correctly.
- Wired SSH agent callbacks and fallback default credentials in `delete_remote_branch` to support authenticated push deletions on real remote repositories.
- Explicitly deleted local remote-tracking reference `refs/remotes/<remote>/<branch>` after successful push deletion in `delete_remote_branch` to guarantee UI list consistency on subsequent scans.
- Centralized and secured date parsing in the Vue/TS desktop frontend to handle RFC3339/ISO-8601 date strings from the Rust backend with timezone offsets or whitespace separators reliably, preventing browser-specific `Invalid Date` errors.
- Fixed silent swallowing of snapshot loading errors in `SqliteHistoryStore` by propagating failures using the `?` operator, and added a comprehensive suite of unit tests for actual SQLite-based snapshot storage and retrieval.



## [0.1.1] — 2026-07-11

### Changed

- Bumped application version to v0.1.1.

## [0.1.0-beta.1] — TBD

### Added

- **Core library** (`gitpurge-core`): hexagonal architecture with domain model,
  port traits (GitBackend, SecretStore, HistoryStore, ReportSink, Clock,
  ProgressSink), and Engine facade with full API surface (scan, plan, backup,
  execute, restore, diff, show_tree, report, history).
- **Backup & Restore Subsystem** (Phase 2):
  - Completed `verify.rs` for snapshot manifest, reference tip, and commit reachability validation.
  - Completed `prune.rs` supporting `RetentionPolicy` rules and physical resource reclaiming.
  - Completed `restore.rs` implementing local ref fetching back to source repositories with `SAFE-06` guardrails.
  - Completed `guard.rs` facilitating automatic pre-op backup (`SAFE-04`) and callback-driven auto-restore on mutation failure (`SAFE-05`).
- **CLI Commands** (Phase 3):
  - Clap skeleton supporting global flags, XDG config routing, human-friendly tables, and raw `--json` envelopes.
  - Verbs: `repo add/list/remove/set-default`, `scan`, `plan`, `delete`, `archive`, `backup`, `restore`, `diff`, `show`.
  - Stubs: `report`, `history`, `auth`, `ui`, `completions`, `install-cli`.
  - TTY-aware confirmation prompts for standard/strong mutation gates.
- **Desktop UI Subsystem** (Phase 4):
  - Completed scaffolding of Tauri v2 backend with Vue 3, Vite, TypeScript, and Pinia.
  - Implemented 26 Tauri command handlers wrapping `gitpurge-core::Engine` with progress events and tokio cancellation.
  - Added architecture guard test ensuring no direct references to git/db dependencies in the desktop crate.
  - Added Makefile shorthand commands (`desktop-dev`, `desktop-build`, `desktop-test`) for Tauri desktop development, build, and test workflows.
  - Designed One Dark Pro and One Light themes using CSS semantic variables.
  - Built App sidebar navigation shell.
  - Created Pinia store managing repository lists, active scan details, and execution tasks.
  - Built Dashboard view with native repository addition via `@tauri-apps/plugin-dialog` directory picker.
  - Built Branches Explorer with status/classification badges, sorting, filtering, selection, and direct comparison.
  - Built Plan Preview & Execute flow showing dry-run logs, unmerged safety confirmations, execution progress, and cancellation.
  - Built Backups and Snapshots browser with ref list, integrity verification, pruning, and restore action.
  - Built Compare/Diff view showing ahead/behind count and list of changed files.
  - Built placeholder panels for History & Trends (Phase 5) and Remote Auth manager (Phase 6).
  - Built Settings view configuring theme toggler, stale age threshold, naming regex, protected/excluded globs, and custom backups path.
  - Generated premium high-resolution RGBA icon assets (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`, `icon.icns`) for Tauri.
- **Reporting & Trend Tracking Subsystem** (Phase 5):
  - Designed SQLite schema migrations using database `PRAGMA user_version`.
  - Implemented the production thread-safe `SqliteHistoryStore` in `gitpurge-core` with WAL journal mode and foreign keys enabled.
  - Implemented automatic **PII Email Redaction** replacing author emails with `"[REDACTED]"` in the history store snapshots to ensure data privacy (`SAFE-07`).
  - Added trend comparison metrics computation vs previous and baseline runs in `trends.rs`.
  - Implemented Markdown, JSON, and theme-aware HTML report generators styling HTML with One Dark Pro design tokens.
  - Wired CLI `report` and `history` commands and Tauri IPC methods to backend engine interfaces.
  - Added `test_golden_reports` asserting report layout and content consistency using `insta` snapshot tests.
- **Desktop UI Completion & Enhancements** (Phase 5 & 6):
  - Wired full interactive credentials manager in `AuthView.vue` supporting add, test, remove, and system default SSH agent fallback notification banners.
  - Built `CleanupView.vue` for whole-repository automated branch plan generation, options overrides, dry-run list, unmerged delete safety confirmations, progress tracking, and cancellation.
  - Built `HistoryView.vue` plotting stale branches over time via SVG charts, with report download options.
  - Added text query search dropdown filters and branch swapping in the Compare/Diff view.
  - Added RFC3339 date payload formatting on Rust commands and integrated fallback date parsers in Branches and Backups views.
  - Integrated on-demand branch status report generation and manual backup snapshot creation with duplicate detection checks directly in Branches Explorer.
  - Updated default naming regex to `^(main|master|develop|staging|prod|production|feat/.*|feature/.*|fix/.*|refactor/.*|docs/.*|perf/.*|test/.*|chore/.*|release/.*|hotfix/.*)$` in Rust Core and Vue App settings.
- **Domain model**: Repository, Branch, Commit, Tag, Ref, Classification, Policy,
  Snapshot, Plan, Action, RunReport, Config, and all supporting value objects.
- **Port trait fakes**: FakeGitBackend, FakeSecretStore, FakeHistoryStore,
  FakeReportSink, FakeClock, FakeProgressSink — proving dependency inversion.
- **Configuration**: TOML-based config with XDG/KnownFolders resolution.
- **Error model**: `GitPurgeError` enum with `miette` diagnostic integration.
- **CI**: GitHub Actions workflow (fmt, clippy, nextest, cargo-deny).
- **Project scaffolding**: workspace layout, delivery phases, conventions,
  architecture spec, safety model, 16 specification documents.

### Changed

- MSRV bumped from 1.82 → 1.88 (required by `time`, `home`, `icu` crate
  ecosystem).
- License changed from dual MIT/Apache-2.0 to Apache-2.0.

[Unreleased]: https://github.com/MohamedGamil/git-purge/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/MohamedGamil/git-purge/compare/v0.1.0-beta.1...v0.1.1
[0.1.0-beta.1]: https://github.com/MohamedGamil/git-purge/releases/tag/v0.1.0-beta.1
