# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- **Desktop UI Foundations** (Phase 4):
  - Completed scaffolding of Tauri v2 backend with Vue 3, Vite, TypeScript, and Pinia.
  - Implemented 26 Tauri command handlers wrapping `gitpurge-core::Engine` with progress events and tokio cancellation.
  - Added architecture guard test ensuring no direct references to git/db dependencies in the desktop crate.
  - Designed One Dark Pro and One Light themes using CSS semantic variables.
  - Built App sidebar navigation shell and mockup dashboard verifying Tauri IPC connectivity.
  - Generated premium high-resolution RGBA icon assets (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`, `icon.icns`) for Tauri.
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

[Unreleased]: https://github.com/MohamedGamil/git-purge/compare/v0.1.0-beta.1...HEAD
[0.1.0-beta.1]: https://github.com/MohamedGamil/git-purge/releases/tag/v0.1.0-beta.1
