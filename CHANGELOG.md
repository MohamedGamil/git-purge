# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Implemented global collapsible active & recent cleanup tasks registry panel inside Repository Cleanup screen (P13-T7).
- Added view reconnect support inside the active cleanups list, letting users navigate back to the ongoing execution progress screen of any running task.
- Implemented global toast notification system in desktop frontend using Pinia `toastStore` and `ToastContainer` UI, replacing all browser `alert()` and dialogs across the views (P13-T8).
- Implemented `<ModalDialog />` component styled with glassmorphism to replace native `confirm()` prompts, featuring keyboard accessibility, a click-to-copy confirmation token, and row-level delete/archive confirmations (P13-T9).
- Added interactive regex allowed naming check in Settings view with UI validation border indicators, compile error displays, and compilation checks in the `settings_save` Tauri backend IPC command (P13-T11).

### Changed

### Fixed

- Resolved indefinite loading state/freeze after executing a cleanup action by guaranteeing the store clears its `isExecuting` state on Tauri command completion.
- Fixed a repository lock hang by skipping auto plan generation on Cleanup View mount if a background task is executing or if a plan already exists.
- Resolved TypeScript compilation type constraints (`ClientPlan`) and cleaned unused declarations/imports in Vue views and store files to guarantee clean production compilation builds (via `vue-tsc`).

## [0.4.0] — 2026-07-17

### Added

- Implemented Vue Pinia stores (`useBranchesStore`, `useBackupsStore`, `useHistoryStore`, `useSettingsStore`) in `apps/desktop/src/stores/` with comprehensive unit and interaction tests to centralize reactive state and Tauri IPC API calls (P13-T1).
- Extracted standalone `show` subcommand to `crates/gitpurge-cli/src/cmd/show.rs` separate from the legacy `diff` subcommand (P13-T6).
- Added `scripts/verify-release.sh` to automate validation of minisign signatures and SHA256 checksums in package releases (P13-T4).
- Added context menu (`contextmenu`) and key combination listener intercepts in `apps/desktop/src/main.ts` to disable developer tools shortcuts and reloading in production builds (P13-T10).
- Implemented `history import --legacy-json` CLI command and history store backend to import legacy SQLite metrics history with chronological sorting and metrics-hash deduplication (P12-T3).
- Added Criterion benchmark suite (`crates/gitpurge-core/benches/engine_bench.rs`) measuring scan, plan, and report operations on a 500-branch repository, with optimized configuration settings for fast execution (P12-T5).
- Expanded `SECURITY.md` with threat model, STRIDE mitigations, trust boundaries, sandboxing details, and safety invariants SAFE-01 to SAFE-07 (P12-T7).
- Implemented multi-threaded branch deletions and archiving utilizing `std::thread::scope` and `mpsc` channels with thread-safe progress ticketing via `AtomicUsize` (P12-T8).
- Expanded CI matrix workflow to run Cargo checks, clippy, and nextest test suites across `ubuntu-20.04`, `ubuntu-latest`, `macos-latest`, and `windows-latest` platforms, plus a Linux-based Vitest job for frontend desktop tests.
- Implemented `TrendDiff` domain structures, `compare_scans` delta algorithms, and wired the `Engine::trends()` method to compute branch deltas, added/removed branches, and daily branch merge velocity over time.
- Integrated `insta` golden-file rendering checks (`report_golden.rs`) verifying Markdown audit/trend, JSON, and HTML report formatting against static mock run fixtures.
- Integrated `proptest` property-testing coverage inside policy modules (`age.rs`, `naming.rs`, and `protection.rs`) validating parser, glob match, and naming evaluator behaviors under arbitrary randomized inputs.
- Split the monolithic `apps/desktop/src-tauri/src/commands.rs` into modular, domain-specific command modules under a new `commands/` directory (`mod.rs`, `repo.rs`, `branch.rs`, `backup.rs`, `history.rs`, `auth.rs`, `settings.rs`, `system.rs`, `tests.rs`), keeping files under 500 lines.
- Added comprehensive Tauri command integration testing using `tauri::test::mock_app()` to verify settings, repo CRUD, and backup prune/list IPC command behaviors.
- Wired the CLI `auth add` and `auth remove` commands to configure credential metadata in `config.toml` via `AuthConfig`/`CredentialMetadata` models, persisting it to disk on configuration updates.
- Refactored `lib.rs` by splitting the monolithic 1.8K lines of engine code into modular sub-modules under `crates/gitpurge-core/src/engine/` (`core.rs`, `auth.rs`, `scan_plan.rs`, `backup.rs`, `execute.rs`, `git_ops.rs`, `report_history.rs`, `tests.rs`), keeping `lib.rs` under 200 lines (now 77 lines) and all modular files under 500 lines.
- Priority-ordered `CredentialResolver` supporting CLI overrides, OS Keyring, fallback file store, environment variables, and interactive prompts, along with host/URL/glob specificity matching.
- Native `git2` credential callback bridge in `git2_backend.rs` routing remote network actions to the `CredentialResolver`.
- Wrapper `FallbackSecretStore` providing automatic degradation to the encrypted file store fallback on headless Linux/CI systems where the native OS keyring daemon is missing.
- Credential metadata config structs (`AuthConfig`, `CredentialMetadata`) to represent credential matching specifications in `config.toml`.
- Encrypted file `SecretStore` adapter (`FileSecretStore`) using AES-256-GCM encryption and Argon2id key derivation, with a fallback when a native OS keyring is not available or desired.
- Standalone branch deletion orchestration module (`action/delete.rs`), refactoring core deletion logic out of the main `lib.rs` engine to prepare for further structural consolidation.
- Keyring `SecretStore` adapter (`KeyringSecretStore`) using the OS keychain to securely store credential payloads, accompanied by a secure JSON index for list and test queries.
- E2E CLI exit-code verification tests asserting stable exit codes (SUCCESS, USAGE, CONFIG, NOT_FOUND) mapped from internal error variants.
- Line-coverage check utilizing `cargo-llvm-cov` to the CI pipeline with a 50% line-coverage threshold gate, along with a corresponding local `coverage` Makefile target.
- Regression tests asserting secret material redaction in both `Credential` and `KeyringSecretStore` structures under `SAFE-07` guidelines.
- Snapshot testing for all major CLI commands (`repo list/add/show`, `scan`, `plan`, `delete`, `backup list`, `history`, `diff`, `show`, `completions`) in both human-readable comfort tables and JSON formats.
- Dedicated workspace round-trip integration test validating the `scan` -> `plan` -> `backup` -> `execute` -> `restore` sequence.
- Comprehensive regression testing verifying secret material redaction in credential debug outputs under `SAFE-07` rules.

### Changed

- Migrated all desktop frontend views (`App.vue`, `BranchesView.vue`, `BackupsView.vue`, `HistoryView.vue`, `PlanView.vue`, `CleanupView.vue`, `DiffView.vue`, `SettingsView.vue`) to use central Pinia stores instead of direct IPC API calls (P13-T1).
- Updated minisign signature generation output to `.sig` files in the GitHub release workflow (P13-T4).
- Formally amended `ADR-0002` (Hybrid git engine) to drop the system `git` CLI shell-out fallback.
- Transitioned `gitpurge-core` from `#![forbid(unsafe_code)]` to `#![deny(unsafe_code)]` with a single audited exemption for `libgit2` global timeout configuration per ADR-0006.
- Updated conventions, project guides, and task lists to align with the Apache-2.0 only license model.

### Fixed

- Reverted the default SSH fallback credential authentication in `git2_backend.rs` to the proven historical implementation using SSH agent first, then defaulting to system Git credentials, fixing a regression where complex key file scanning broke remote branch deletions on self-hosted Azure DevOps/TFS servers.
- Sandboxed the `install-cli` CLI command integration test to target a temporary directory, avoiding conflicts with the host system's actual binary path.
- Prevented potential secret leakage by implementing a redacted `std::fmt::Debug` manually for the `StoredCredential` type.

## [0.3.8] — 2026-07-13

### Fixed

- Updated Linux desktop build runner to `ubuntu-22.04` to ensure dynamic linking against GLIBC 2.35, making the built Debian, RPM, and AppImage installers fully backward-compatible with Ubuntu 22.04+ (avoiding `GLIBC_2.38 not found` errors).

## [0.3.7] — 2026-07-13

### Fixed

- Enabled Tauri installer bundling in `tauri.conf.json`, staged flat compiled installers in `release.yml` to resolve empty artifact upload warnings, and implemented repackaging logic to bundle both CLI and Desktop apps together for Linux, Windows, and macOS.

## [0.3.6] — 2026-07-13

### Fixed

- Resolved Minisign signature generation failure in release workflow by writing the secret key to a temporary seekable file instead of using non-seekable Bash process substitution.

## [0.3.5] — 2026-07-13

### Fixed

- Resolved database lock errors (`database is locked`) when running integration tests in parallel by ensuring CLI tests write isolated configurations to their respective temporary directories.
- Fixed Windows `.zip` packaging failure in release workflow by integrating fallback support for `7z` and PowerShell's `Compress-Archive` cmdlets inside the packaging script.

## [0.3.4] — 2026-07-13

### Changed

- Updated `crates/gitpurge-core/Cargo.toml` to use target-specific `git2` feature configurations, enabling `vendored-openssl` and `vendored-libgit2` on Unix/macOS for static/self-contained compilation, while keeping only `vendored-libgit2` on Windows to leverage native Schannel and avoid OpenSSL compile overhead.
- Modified `.github/workflows/release.yml` to compile targets natively using standard `cargo` on macOS and Windows runners, limiting `cross` compilation to Linux runners.
- Explicitly configured the `shell: bash` environment for CLI packaging steps on all runners to ensure cross-platform compatibility.

### Fixed

- Resolved macOS target compilation failure (`x86_64-apple-darwin`) in the release workflow due to missing host OpenSSL developer files.
- Removed redundant and invalid `apps/desktop/pnpm-workspace.yaml` file that was causing pnpm workspace resolution errors (`packages field missing or empty`) on desktop builds.

## [0.3.3] — 2026-07-12

### Added

- Implemented `ui`, `completions`, `install-cli`, and `auth` (add, list, remove, test) subcommands.
- Added integration tests for completions script generation, install-cli dry-run, ui absence error handling, and E2E auth flows.
- Added a new `backup_path` column to the `snapshots` database table via a version 2 schema migration.
- Added automatic database `backup_path` backfill migration on startup for older snapshots to ensure backwards compatibility.
- Added backups path resolution fallback lookup in both `sqlite` history store and bare mirror manager to support older snapshots created in nested or non-nested layouts.
- Added automatic migration of the old database and backups from the XDG data directory (`~/.local/share/git-purge`) to the new unified data directory (`~/.gitpurge`) on startup.

### Changed

- Renamed the short option for the `exclude` flag in `SelectionFlags` from `-e` to `-x` to resolve conflict with the global `-e/--execute` flag.
- Removed duplicate `unmerged` argument from the `Delete` subcommand in favor of using `SelectionFlags.unmerged`.
- Unified the operations log directory from `~/.git-purge` to `~/.gitpurge`.
- Updated the default backups root directory fallback to nest inside a `backups` subfolder (`~/.gitpurge/backups`) by default, while custom backups directories are used directly.
- Updated the Settings screen UI to state the default backups root directory path when left blank.

### Fixed

- Fixed CI `cargo-deny` wildcard dependency error by adding explicit `version` fields to `gitpurge-core` path dependencies in CLI and Desktop crate manifests.
- Removed stale `Unicode-DFS-2016` license allowance from `deny.toml` (no dependency uses it anymore).
- Suppressed transitive duplicate crate warnings from `gix v0.66` dependency tree via `skip-tree` in `deny.toml`.
- Added advisory ignore entries for known RUSTSEC advisories in `gix v0.66`, `git2 v0.19`, and `number_prefix v0.4` where no safe upgrade is available.
- Fixed Ubuntu/GNOME taskbar showing binary name `gitpurge-desktop` instead of "Git Purge" by setting the GTK application name on Linux.

## [0.3.2] — 2026-07-12

### Added

- Added shorthand local make targets (`bundle-cli`, `bundle-desktop`, `bundle`) in the root `Makefile` for compiling and packaging releases.
- Added a packaging helper script `ci/package-tarball.sh` to package CLI release binaries with licenses, README, and installer wrappers (`install.sh`/`install.ps1`).
- Added a GitHub Actions release workflow `.github/workflows/release.yml` triggered on tags to automate cross-platform compilation and release artifact generation.
- Added unit tests for default home-folder based data directory fallback.

### Changed

- Updated the default custom data directory resolver to fall back to the `.gitpurge` directory in the user's home folder (`~/.gitpurge/` on Linux/macOS and `%USERPROFILE%\.gitpurge\` on Windows) when no custom directory is configured, replacing OS-specific local project data directories.

## [0.3.1] — 2026-07-12

### Fixed

- Fixed cargo-deny schema validation errors in `deny.toml` by renaming `wildcard-dependencies` to `wildcards` and `allowed-by` to `wrappers`.
- Fixed Rust code formatting violations in Tauri command bridge.

## [0.3.0] — 2026-07-12

### Added

- Added expandable past operations/executions log in the History view, listing past purge/scan runs, actors, modes, and affected branch details.
- Added past executions table output to the CLI `git-purge history` command with configurable limit support.
- Added paginated SQL queries (`LIMIT`/`OFFSET`) for past executions log retrieval to ensure optimal database performance.
- Added report preview loading spinners and animations in both Branches Explorer and History View screens.
- Added remote-prefix-aware regex and text search capability in the Branches Explorer UI, allowing patterns (e.g. `^AZ-`) to match remote branches by testing against the short branch name (remote prefix stripped) as well as the full display name.
- Added Settings Export & Import capability in the Settings UI view, allowing users to save and load `config.toml` configurations dynamically using native file save/open dialogs.
- Added date-time display format customization in Settings (defaulting to "YYYY-MM-DD h:m a"), allowing users to standardize timestamps in backups, history runs, and branch metadata lists.
- Added Web App Manifest (`manifest.json`) and favicon/apple-touch-icon links to the desktop view templates.
- Added native default browser URL opening bridge using a new backend Tauri command (`open_url`) to handle external author links safely.

### Changed

- Standardized the report generation interface in the History View to use the new multi-tab Markdown preview modal matching the Branches Explorer.
- Replaced native OS-specific emojis throughout the desktop UI with SVG-based Lucide icons styled via theme-aware CSS variables.
- Enhanced the collapsable backup snapshots list in the Backups view with left-accent active borders, fade-in transitions, layout height stability, accessible zebra-striped references table, and a dedicated Restore button with RotateCcw icon.
- Replaced primary UI font with `'Google Sans'` and secondary monospace font with `'JetBrains Mono'`.
- Replaced the generic SVG brand logo with a newly designed, 17.5% rounded squircle brand icon regenerated from `polished_app_icon.png`.
- Updated tauri bundle configuration to explicitly package all standard platform icons (ICO, ICNS, and PNG resolutions) to ensure tray/taskbar consistency.
- Updated footer copyright notice with version tag, dynamic current year, and author link.
- Updated sidebar logo header to act as a router link navigating to the Dashboard view.
- Standardized all repository selectors and theme toggle dropdowns to use `8px` (`var(--radius-sm)`) corner radius.
- Applied global text selection disablement (`user-select: none`) to the sidebar navigation panel, all view headers, and heading tags (`h1`-`h6`) to prevent accidental highlight.

### Fixed

- Fixed a bug where backup snapshots captured 0 references when remote-prefixed branches (e.g. `origin/feat/1`) were selected.
- Fixed target ambiguity on branch restoration by introducing an optional `originalRef` matching parameter, ensuring correct target branch resolution when local and remote branches share a short branch name.
- Fixed a repository target mismatch in snapshot verification where the Tauri command hardcoded the repository ID to `"default"`, dynamically resolving the repository from the snapshot metadata instead.
- Fixed a bug where planning dry-runs on remote branch selections resulted in `0` actions by reusing the parsed remote name in the planning engine rather than trying to split the branch short name.

## [0.2.0] — 2026-07-12

### Added

- Added a new integration test suite `test_sqlite_history_store_runs_and_trends_flow` in `sqlite.rs` to validate SQLite adapter operations, including runs logging, PII email redaction, metrics deduplication, and trend history APIs.
- Added configured staleness age threshold display to the header subtitle of the Branches Explorer UI.
- Added regex-based branch search capabilities to the Branches Explorer search bar, with an automatic fallback to normal case-insensitive substring search on compilation errors.
- Added a thread-safe `get_remotes` method to the core `Engine` in `gitpurge-core` to retrieve configured repository remotes dynamically using libgit2, keeping external adapter layers thin.
- Introduced `remote` and `upstream` fields to the shared `Classification` model, allowing git metadata to propagate transparently through Tauri commands to the frontend.
- Introduced `DESIGN.md` defining brand personality, layout grids, components, typography scales, and shape design requirements.
- Implemented and mapped design color tokens for both dark (default) and light themes.
- Enabled remote branch deletion selection, planning, and execution with premium warning prompts in both the Branches view and Plan execution screen.
- Implemented native OS save dialog integration utilizing `tauri-plugin-dialog` to bypass webview Content Security Policy (CSP) limitations on Blob downloads.
- Added branch locality (local vs remote) markers to backup references table in Backups view.
- Added an "Auto Fetch" checkbox (checked by default) in the Branches Explorer UI, core engine scan options, and CLI command flags to perform automatic remote fetch and pruning of all remotes before classifying branches.

### Changed

- Modified the Branches Explorer to persist and restore filter selections (search, locality, freshness, merge, protection, naming, and sorting) per repository in `localStorage`.
- Enhanced remote branch display, planning, and execution to dynamically resolve the branch's actual remote name (e.g. `upstream`, `origin`, or custom remotes) instead of hardcoding `"origin"`.
- Updated `computeBranchesHash` in the Branches Explorer UI to strip common remote prefixes (like `origin/` and `upstream/`) dynamically when calculating local branch hashes for duplicate backup warnings.
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

- Fixed a bug where remote branch deletions were hardcoded to push/delete on `"origin"`, dynamically resolving the remote server from reference structures instead.
- Fixed Tauri adapter `plan` and `delete_branches` commands to dynamically parse the remote and short branch name from selected reference paths.
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

[Unreleased]: https://github.com/MohamedGamil/git-purge/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/MohamedGamil/git-purge/compare/v0.3.8...v0.4.0
[0.3.8]: https://github.com/MohamedGamil/git-purge/compare/v0.3.7...v0.3.8
[0.3.7]: https://github.com/MohamedGamil/git-purge/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/MohamedGamil/git-purge/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/MohamedGamil/git-purge/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/MohamedGamil/git-purge/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/MohamedGamil/git-purge/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/MohamedGamil/git-purge/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/MohamedGamil/git-purge/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/MohamedGamil/git-purge/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/MohamedGamil/git-purge/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/MohamedGamil/git-purge/compare/v0.1.0-beta.1...v0.1.1
[0.1.0-beta.1]: https://github.com/MohamedGamil/git-purge/releases/tag/v0.1.0-beta.1
